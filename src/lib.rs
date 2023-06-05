use std::{io::{Stdout, Write}, time::Duration, thread, iter::Peekable};
use crossterm::{style::{Stylize, self, Color}, cursor, queue, execute};
use rand::{seq::SliceRandom, Rng};
use rand_chacha::ChaCha8Rng;
use simple_simplex::NoiseConfig;
pub mod branch;
use branch::{BonsaiBranch, Direction, BonsaiStep};
pub mod point;
use point::Point;
pub mod appearance;
use appearance::{TreeAppearance, BaseType};


const BROWN: Color = Color::Rgb {r: 142, g: 44, b: 19};
const ROSE: Color = Color::Rgb { r: 252, g: 212, b: 251 };
type Writer = Stdout;
pub type RNG = ChaCha8Rng;
//pub type IterOveru16Point<'a> = std::iter::Rev<std::slice::Iter<'a, Point<i16>>>;
pub type IterOveru16Point = std::iter::Rev<std::vec::IntoIter<Point<i16>>>;


#[derive(Debug, Clone)]
struct Growstate {
    pub branch_steps: Peekable<std::vec::IntoIter<BonsaiStep>>,
    pub branch: Option<BonsaiBranch>,
    pub last_branch_height: i16,
    pub last_branch_dir: Direction,
    pub grow_leaves: Option<IterOveru16Point>,
    pub num_leaves_left: usize,
}
impl Growstate {
    pub fn grow_leaves_for(branch_steps: Peekable<std::vec::IntoIter<BonsaiStep>>, branch: BonsaiBranch, leaf_positions_count: usize, last_branch_height: i16, last_branch_dir: Direction) -> Self {
        let leafs: IterOveru16Point = {
            let mut positions: Vec<Point<i16>> = branch.steps.iter().rev()
            .filter_map(|step| Some(step.pos))
            .collect();
            positions.truncate(leaf_positions_count);
            positions.into_iter().rev()
        };
        
        Growstate {
            branch_steps,
            branch: Some(branch),
            last_branch_height,
            last_branch_dir,
            grow_leaves: Some(leafs),
            num_leaves_left: 0,
        }
    }

    pub fn search_branch_pos(branch_steps: Peekable<std::vec::IntoIter<BonsaiStep>>, last_branch_height: i16, last_branch_dir: Direction) -> Growstate {
        Growstate {
            branch_steps,
            branch: None,
            last_branch_height,
            last_branch_dir,
            grow_leaves: None,
            num_leaves_left: 0,
        }
    }

    pub fn grow_branch(branch_steps: Peekable<std::vec::IntoIter<BonsaiStep>>, branch: &mut BonsaiBranch, last_branch_height: i16, last_branch_dir: Direction) -> Growstate {
        Growstate {
            branch_steps,
            branch: Some(branch.clone()),
            last_branch_height,
            last_branch_dir,
            grow_leaves: None,
            num_leaves_left: 0,
        }
    }
}

#[derive(Debug, Clone)]
enum TreeState {
    /// Grow out the bonsai trunk
    GrowTrunk,
    /// Grow the branches out of the trunk
    /// 
    /// Args: `[<trunk steps iterator>, <current branch>, <last branch height>, <last branch direction>, <grow leaves>, <num leaves left>]`
    GrowBranches(Growstate),
}


pub struct BonsaiTree {
    pub noise: NoiseConfig,
    pub rng: RNG,
    pub seed: u64,
    pub stdout: Writer,
    pub width: i16,
    pub height: i16,
    branches: Vec<BonsaiBranch>,
    pub time_multiplier: f32,
    pub appearance: TreeAppearance,
    current_state: TreeState,
}

impl BonsaiTree {
    /// Creates a new randomized tree with the given values
    pub fn new(noise: NoiseConfig, mut rng: RNG, seed: u64, stdout: Writer, width: i16, height: i16, trunk_width: u8) -> BonsaiTree {
        let appearance = TreeAppearance::randomize(&mut rng, trunk_width);
        BonsaiTree {
            noise,
            rng,
            seed,
            stdout,
            width,
            height,
            branches: vec![],
            // Default value, gets overwritten in main.rs by the cmd args
            time_multiplier: 1.0,
            appearance,
            current_state: TreeState::GrowTrunk,
        }
    }


    pub fn step(&mut self) {
        let state = self.current_state.clone();
        match state {
            TreeState::GrowTrunk => {
                if self.branches.is_empty() {
                    let baseheight = self.draw_base();
                    let w = self.appearance.trunk_width;
                    // Center the tree trunk in the horizontal axis and above the base (plant pot)
                    let start = Point { x: self.width / 2 - (w as f32 / 2.0 as f32).round() as i16, y: self.height - baseheight as i16};
                    
                    let mut trunk = BonsaiBranch::new_trunk(start, w, false, &mut self.rng);
                    self.step_branch(&mut trunk);
                    self.branches.push(trunk);
                }
                let mut trunk = &mut self.branches[0];
                if !self.step_branch(trunk) {
                    self.current_state = TreeState::GrowBranches(
                        Growstate::grow_leaves_for(
                            trunk.steps.clone().into_iter().peekable(),
                            trunk,
                            self.appearance.leaf_count,
                            0,
                            Direction::Up
                        )
                    );
                }
                //self.branches[0] = trunk;
            },
            TreeState::GrowBranches(
                mut growstate) => {
                // if there is currently no branch to be grown, find the next position to grow a branch
                
                while growstate.branch.is_none() {
                    let step = growstate.branch_steps.next();
                    if step.is_none() {
                        break;
                    }
                    let step = step.unwrap();

                    let mut ratio = 1.0 - (step.pos.y as f32 / (self.height - 1) as f32);
                    ratio = ratio.clamp(0.0, 1.0);
                    
                    let dir = vec![Direction::Left, Direction::Right]
                        .choose(&mut self.rng)
                        .unwrap()
                        .clone();
                    let min_width_between_branches = 2;
                    let min_ratio = 0.35;
                    // Only spawn branch if it has some distance to the other branches and its towards the top
                    if (dir == growstate.last_branch_dir && (step.pos.y - growstate.last_branch_height).abs() > min_width_between_branches) || (dir != growstate.last_branch_dir && ratio > min_ratio && self.rng.gen_bool(ratio as f64)) {
                        let mut b_width = min_width_between_branches;
                        if step.width <= 2 {
                            b_width = 1;
                        };
                        
                        let mut branch = BonsaiBranch::new_branch(step.pos, b_width as u8, dir.clone(), false, &mut self.rng);
                        self.current_state = TreeState::GrowBranches(Growstate::grow_branch(growstate.branch_steps.clone(), &mut branch, step.pos.y, dir));
                        self.branches.push(branch);
                    }
                }
                let branch = match &self.current_state {
                    TreeState::GrowBranches(growstate) => growstate.branch.clone(),
                    _ => None,
                };
                
                // if we are currently growing a branch, grow/ step that branch
                if let Some(mut branch) = branch {
                    // If the branch should now grow leaves
                    if growstate.grow_leaves.is_some() {
                        let mut leaf_positions = growstate.grow_leaves.unwrap();
                        //println!("Grow leaves: {:?}, Leaves left: {}", leaf_positions, growstate.num_leaves_left);
                        let leaf_pos = match growstate.num_leaves_left {
                            0 => {
                                growstate.num_leaves_left = self.get_num_leaves();
                                //self.draw((1, 1), "                                                                                        ", Color::Red);
                                //self.draw((1, 1), "Getting next leaf", Color::Red);
                                leaf_positions.next()
                            },
                            _ => {
                                //self.draw((1, 1), "                                                                                        ", Color::Red);
                                //self.draw((1, 1), format!("Getting existing leaf {:?}", leaf_positions.clone().collect::<Vec<Point<i16>>>()).as_str(), Color::Red);
                                leaf_positions.nth(0)
                            },
                        };

                        //println!("Grow leaves leaf pos: {:?}", leaf_pos);
                        if let Some(pos) = leaf_pos {
                            let leaf = &self.appearance.get_leaf_string(&mut self.rng);
                            
                            let rand_x = self.rng.gen_range(-self.appearance.leafshape_x.x..=self.appearance.leafshape_x.y);
                            let rand_y = self.rng.gen_range(-self.appearance.leafshape_y.x..=self.appearance.leafshape_y.y);
                            let mut new_pos = pos + Point::from((rand_x, rand_y));
                            new_pos.x = std::cmp::max(3, new_pos.x);
                            new_pos.y = std::cmp::max(3, new_pos.y);
                            self.draw((new_pos.x as u16, new_pos.y as u16), leaf, self.appearance.leaf_color);
                            
                            self.current_state = TreeState::GrowBranches(Growstate {
                                branch_steps: growstate.branch_steps,
                                branch: growstate.branch,
                                last_branch_height: growstate.last_branch_height,
                                last_branch_dir: growstate.last_branch_dir,
                                grow_leaves: Some(leaf_positions),
                                num_leaves_left: growstate.num_leaves_left - 1,
                            })
                        } else {
                            // if all leaves have been grown, go back to searching for a new branch position
                            self.current_state = TreeState::GrowBranches(Growstate::search_branch_pos(growstate.branch_steps, growstate.last_branch_height, growstate.last_branch_dir))
                        }
                    } else {
                        let did_grow = self.step_branch(&mut branch);
                        // Branch finished growing, now grow leaves
                        if !did_grow {
                            self.current_state = TreeState::GrowBranches(
                                Growstate::grow_leaves_for(
                                    growstate.branch_steps,
                                    branch,
                                    self.appearance.leaf_count,
                                    growstate.last_branch_height,
                                    growstate.last_branch_dir
                                )
                            );
                        } else {
                            // continue growing
                            self.current_state = TreeState::GrowBranches(Growstate::grow_branch(growstate.branch_steps, &mut branch, growstate.last_branch_height, growstate.last_branch_dir))
                        }
                    }
                }
            },
        }

        let seed_draw_height = self.height.saturating_sub(2);
        self.draw((1, seed_draw_height as u16), format!("Seed: {}", self.seed).as_str(), Color::DarkGrey);
        self.flush();
    }


    /// The core of the program. Basically every part of the bonsai tree is handled as a branch
    /// with a direction (even the trunk)
    pub fn step_branch(&mut self, branch: &mut BonsaiBranch) -> bool {
        let last_steppos = branch.steps.last().unwrap().pos;
        let ratio = match branch.direction {
            Direction::Up =>  1.0 - (last_steppos.y as f32 / (self.height - 1) as f32),
            Direction::Left =>  1.0 - (last_steppos.x as f32 / (self.width - 1) as f32),
            Direction::Right =>  last_steppos.x as f32 / (self.width - 1) as f32,
            _ => 0.0,
        };

        let did_grow = branch.step(&self.noise, &mut self.rng, branch.shape.width_loose_chance, ratio);
        if branch.shape.width_loose_chance > branch.shape.min_width_loose_chance {
            branch.shape.width_loose_chance *= branch.shape.width_loose_ratio;
        } else {
            branch.shape.width_loose_chance = branch.shape.min_width_loose_chance;
        }
        branch.shape.width_loose_chance = (branch.shape.width_loose_chance * 10.0).round() / 10.0;

        self.draw_step(branch.steps.last().unwrap(), branch.direction.clone());

        did_grow
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


    pub fn get_num_leaves(&mut self) -> usize {
        let l = self.rng.gen_range(5..=10+(3 * self.appearance.trunk_width_bonus)) as usize;
        //println!("get_num_leaves: {}", l);
        l
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
