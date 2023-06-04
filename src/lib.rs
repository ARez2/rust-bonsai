use std::{io::{Stdout, Write}, time::Duration, thread};
use crossterm::{style::{Stylize, self, Color}, cursor, queue, execute};
use rand::{seq::SliceRandom, Rng};
use rand_chacha::ChaCha8Rng;
use simple_simplex::NoiseConfig;
pub mod branch;
use branch::{BonsaiBranch, Direction, BonsaiStep};
pub mod point;
use point::Point;
pub mod appearance;
use appearance::{TreeAppearance, LeafType, BranchShape, BaseType};


const BROWN: Color = Color::Rgb {r: 142, g: 44, b: 19};
const ROSE: Color = Color::Rgb { r: 252, g: 212, b: 251 };
type Writer = Stdout;
pub type RNG = ChaCha8Rng;


pub struct BonsaiTree {
    pub noise: NoiseConfig,
    pub rng: RNG,
    pub stdout: Writer,
    pub width: i16,
    pub height: i16,
    branches: Vec<BonsaiBranch>,
    pub time_multiplier: f32,
    pub appearance: TreeAppearance,
}

impl BonsaiTree {
    /// Creates a new randomized tree with the given values
    pub fn new(noise: NoiseConfig, mut rng: RNG, stdout: Writer, width: i16, height: i16, trunk_width: u8) -> BonsaiTree {
        let appearance = TreeAppearance::randomize(&mut rng, trunk_width);
        BonsaiTree {
            noise,
            branches: vec![],
            rng,
            stdout,
            width,
            height,
            // Default value, gets overwritten in main.rs by the cmd args
            time_multiplier: 1.0,
            appearance,
        }
    }


    /// Starts the growing of the tree
    pub fn grow(&mut self) {
        let baseheight = self.draw_base();
        self.grow_trunk(baseheight);
        self.grow_branches();
        self.flush();
    }


    /// Helper function to draw the trunk
    pub fn grow_trunk(&mut self, baseheight: usize) {
        let w = self.appearance.trunk_width;
        // Center the tree trunk in the horizontal axis and above the base (plant pot)
        let start = Point { x: self.width / 2 - (w as f32 / 2.0 as f32).round() as i16, y: self.height - baseheight as i16};
        self.grow_branch(start, Direction::Up, w, self.appearance.trunk_shape);
    }


    /// Helper function to draw all the other branches
    pub fn grow_branches(&mut self) {
        if self.branches.is_empty() {
            return;
        };
        let trunk_steps = self.branches[0].steps.clone();
        let mut last_branch_height = 0;
        let mut last_branch_dir = Direction::Up;
        for step in trunk_steps {
            let mut ratio = 1.0 - (step.pos.y as f32 / (self.height - 1) as f32);
            if ratio < 0.0 {
                ratio = 0.0;
            } else if ratio > 1.0 {
                ratio = 1.0;
            };
            
            let dir = vec![Direction::Left, Direction::Right]
                .choose(&mut self.rng)
                .unwrap()
                .clone();
            let default_width = 2;
            let min_ratio = 0.35;
            // Only spawn branch if it has some distance to the other branches and its towards the top
            if (dir == last_branch_dir && (step.pos.y - last_branch_height).abs() > default_width) || (dir != last_branch_dir && ratio > min_ratio && self.rng.gen_bool(ratio as f64)) {
                let mut b_width = default_width;
                if step.width <= 2 {
                    b_width = 1;
                };
                self.grow_branch(step.pos, dir.clone(), b_width as u8, self.appearance.branch_shape);
                last_branch_height = step.pos.y;
                last_branch_dir = dir;
            };
        };
    }


    /// The core of the program. Basically every part of the bonsai tree is handled as a branch
    /// with a direction (even the trunk)
    pub fn grow_branch(
        &mut self,
        start_pos: Point<i16>,
        direction: Direction,
        start_width: u8,
        mut shape: BranchShape,
    ) {
        let mut branch = BonsaiBranch::new(start_pos, direction.clone(), start_width);

        // Simulate the branch growing and handle the width of the branch using the given shape
        for i in (0..self.height - 3).rev() {
            let last_steppos = branch.steps.last().unwrap().pos;
            let ratio = match direction {
                Direction::Up =>  1.0 - (last_steppos.y as f32 / (self.height - 1) as f32),
                Direction::Left =>  1.0 - (last_steppos.x as f32 / (self.width - 1) as f32),
                Direction::Right =>  (last_steppos.x as f32 / (self.width - 1) as f32),
            };
            //let ratio = 1.0 - (branch.steps.last().unwrap().pos.y as f32 / (self.height - 1) as f32);
            let did_grow = branch.step(&self.noise, &mut self.rng, shape.width_loose_chance, ratio);
            if shape.width_loose_chance > shape.min_width_loose_chance {
                shape.width_loose_chance *= shape.width_loose_ratio;
            } else {
                shape.width_loose_chance = shape.min_width_loose_chance;
            }
            shape.width_loose_chance = (shape.width_loose_chance * 10.0).round() / 10.0;

            self.draw_step(branch.steps.last().unwrap(), direction.clone());
            self.flush();
            if did_grow && i > 2 {
                thread::sleep(Duration::from_millis((100 as f32 * self.time_multiplier).round() as u64));
            } else if !did_grow {
                break;
            };
        };
        
        self.grow_leaves(&branch);
        self.branches.push(branch);
    }


    /// Grows/ draws a bunch of leaves at the last few points of a branch
    pub fn grow_leaves(&mut self, branch: &BonsaiBranch) {
        let mut positions: Vec<Point<i16>> = branch.steps.iter()
            .filter_map(|step| Some(step.pos))
            .collect();
        let mut leaf_positions = vec![];
        for pos in positions.iter().rev() {
            if leaf_positions.len() > self.appearance.leaf_count as usize {
                break;
            };
            leaf_positions.push(pos.clone());
        };
        for pos in leaf_positions.iter().rev() {
            let leaf = &self.appearance.get_leaf_string(&mut self.rng);

            self.draw((pos.x as u16, pos.y as u16), leaf, self.appearance.leaf_color);
            let num_leaves = self.rng.gen_range(5..=10+(3 * self.appearance.trunk_width_bonus));
            for _ in 0..num_leaves {
                let rand_x = self.rng.gen_range(-self.appearance.leafshape_x.x..=self.appearance.leafshape_x.y);
                let rand_y = self.rng.gen_range(-self.appearance.leafshape_y.x..=self.appearance.leafshape_y.y);
                let mut new_pos = *pos + Point::from((rand_x, rand_y));
                new_pos.x = std::cmp::max(3, new_pos.x);
                new_pos.y = std::cmp::max(3, new_pos.y);
                self.draw((new_pos.x as u16, new_pos.y as u16), leaf, self.appearance.leaf_color);
            };
            self.flush();
            thread::sleep(Duration::from_millis((50 as f32 * self.time_multiplier).round() as u64));
        }
    }


    /// Depending on the direction, returns a string the looks like the direction
    pub fn get_string_for_dir(&mut self, mut dir: (i16, i16), width: u8) -> String {
        dir.0 = std::cmp::max(std::cmp::min(1, dir.0), -1);
        dir.1 = std::cmp::max(std::cmp::min(1, dir.1), -1);
    
        let set: Vec<char> = match dir {
            // Straight up
            (0, -1) | (0, 0) => vec!['/', '|', '\\'],
            // Up left or down right
            (-1, -1) | (1, 1) => vec!['\\', '~',],
            // Up right or down left
            (1, -1) | (-1, 1) => vec!['/', '~', '\\'],
            // left
            (-1, 0) => vec!['\\', '~', '-', '_', '='],
            // right
            (1, 0) => vec!['/', '~', '-', '_', '='],
            // Stupid dir
            _ => vec!['?']
        };
        let mut result = String::new();
        result.push(set[0]);
        for _i in 0..width {
            result.push(*set.choose(&mut self.rng).unwrap_or(&'?'));
        };
        result
    }


    /// Uses the direction to draw the step of a branch
    fn draw_step(&mut self, step: &BonsaiStep, direction: Direction) {
        let what = self.get_string_for_dir( step.diff, step.width);
        if direction != Direction::Up && step.width <= 1 {
            self.draw((step.pos.x as u16, (step.pos.y + 1) as u16), &what, BROWN);
        } else {
            self.draw((step.pos.x as u16, step.pos.y as u16), &what, BROWN);
        }
    }


    /// Helper function to draw anything on the screen at a specified position
    pub fn draw(&mut self, pos: (u16, u16), what: &str, mut color: Color) {
        let rainbow_col = Color::Rgb { r: 1, g: 1, b: 1 };
        if color == rainbow_col {
            color = *[Color::Rgb { r: 20, g: 0, b: 62 }, Color::DarkBlue, Color::Blue, Color::Green, Color::Yellow, Color::Rgb { r: 100, g: 38, b: 16 }, Color::Red].choose(&mut self.rng).unwrap();
        };

        queue!(self.stdout,
            cursor::MoveTo(pos.0.into(), pos.1.into()),
            style::PrintStyledContent(what.with(color))
        ).unwrap();
    }


    /// Actually renders the string returned by `generate_base()`
    pub fn draw_base(&mut self) -> usize {
        let data = self.generate_base();
        let data_split: Vec<&str> = data.split('\n').collect();
        let data_split: Vec<&str> = data_split.iter()
            .filter_map(|f| if *f == "" {
                None
            } else {
                Some(*f)
            }).collect();
        //println!("{:?}", x);
        let size_x = data_split[0].len();
        let size_y = data_split.len() - 1;

        let mut pos = (
            self.width as u16 / 2 - size_x as u16 / 2,
            self.height as u16 - 1 - size_y as u16
        );

        // Draws the grass of the base
        self.draw(pos, data_split[0], Color::Green);
        //pos.1 += 1;
        for l in 1..data_split.len() {
            //let mut p = pos;
            pos.1 += 1 as u16;
            //execute!(self.stdout, cursor::MoveTo(5, p.1));
            //println!("{:?}", pos);
            self.draw(pos, data_split[l], Color::White);
        };
        //self.flush();
        size_y
    }



    /// Generates a string that is used to draw the base (plant pot)
    pub fn generate_base(&self) -> String {
        let borders = [('\\', '/'), ('(', ')')];
        let border = match self.appearance.base {
            BaseType::LargePot => borders[0],
            BaseType::SmallPot => borders[1],
        };
        let mut w = self.appearance.trunk_width as usize + 10;
        w = std::cmp::max(w, 3);
        let mut base = String::new();
        base.push(' ');
        base.push_str(stretched_str('_', w).as_str());
        base.push('\n');
        base.push(border.0);
        base.push_str(stretched_str(' ', w - 1).as_str());
        base.push(border.1);
        base.push('\n');
        base.push(' ');
        base.push(border.0);
        base.push_str(stretched_str('_', w - 3).as_str());
        base.push(border.1);

        base
    }


    /// Helper function to flush the screen
    pub fn flush(&mut self) {
        self.stdout.flush().unwrap();
    }
}


/// Equivalent of pythons `print("s" * 10)`
pub fn stretched_str(string: char, len: usize) -> String {
    let mut data = String::new();
    for _ in 0..len {
        data.push(string);
    };
    data
}
