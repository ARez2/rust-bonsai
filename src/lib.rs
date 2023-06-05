use std::{io::{Write}};
use crossterm::{style::{Stylize, self, Color}, cursor, queue, execute};
use rand::{seq::SliceRandom, Rng};
use rand_chacha::ChaCha8Rng;
use simple_simplex::NoiseConfig;
pub mod branch;
use branch::{BonsaiBranch, Direction, BranchShape};
pub mod point;
use point::Point;
pub mod appearance;
use appearance::{TreeAppearance};


const BROWN: Color = Color::Rgb {r: 142, g: 44, b: 19};
const ROSE: Color = Color::Rgb { r: 252, g: 212, b: 251 };
pub type RNG = ChaCha8Rng;
pub type Writer = std::io::Cursor<Vec<u8>>;


pub struct BonsaiTree {
    pub noise: NoiseConfig,
    pub rng: RNG,
    pub stdout: Writer,
    pub width: i16,
    pub height: i16,
    branches: Vec<BonsaiBranch>,
    pub appearance: TreeAppearance,
}

impl BonsaiTree {
    /// Creates a new randomized tree with the given values
    pub fn new(noise: NoiseConfig, mut rng: RNG, stdout: Writer, width: i16, height: i16, trunk_width: usize) -> BonsaiTree {
        let appearance = TreeAppearance::randomize(&mut rng, trunk_width);
        
        let baseheight = appearance.get_base(4).lines().count();
        let mut branches = vec![];
        let w = appearance.trunk_width;
        // Center the tree trunk in the horizontal axis and above the base (plant pot)
        let start = Point { x: width / 2 - (w as f32 / 2.0 as f32).round() as i16, y: height - baseheight as i16};
        branches.push(
            BonsaiBranch::new(
                start,
                Direction::Up,
                w,
                BranchShape::default_trunk(),
                BROWN,
                appearance.leaf_count,
                appearance.leaf_type,
                appearance.leaf_color
            )
        );
        BonsaiTree {
            noise,
            branches,
            rng,
            stdout,
            width,
            height,
            appearance,
        }
    }


    pub fn step(&mut self) {
        let mut max_branch_height = 0;
        let mut max_branch_dir = Direction::Up;
        for (idx, branch) in self.branches.iter_mut().enumerate() {
            if idx > 0 && branch.steps[0].pos.y > max_branch_height {
                max_branch_height = branch.steps[0].pos.y;
                max_branch_dir = branch.direction.clone();
            }
            let g = branch.step(
                &self.noise,
                &mut self.rng,
                (self.width, self.height),
                &mut self.stdout);
        }
        let last_trunk_step = self.branches[0].steps.last().unwrap();
        let mut ratio = 1.0 - (last_trunk_step.pos.y as f32 / (self.height - 1) as f32);
        if ratio < 0.0 {
            ratio = 0.0;
        } else if ratio > 1.0 {
            ratio = 1.0;
        };
        let dir = vec![Direction::Left, Direction::Right]
            .choose(&mut self.rng)
            .unwrap()
            .clone();
        let default_width = last_trunk_step.width;
        let min_ratio = 0.35;
        // Only spawn branch if it has some distance to the other branches and its towards the top
        if default_width > 1 && ratio > min_ratio && ratio < 1.0 && ((dir == max_branch_dir && (last_trunk_step.pos.y - max_branch_height).abs() > default_width as i16) || (dir != max_branch_dir && self.rng.gen_bool(ratio as f64))) {
            let mut b_width = default_width;
            if last_trunk_step.width <= 2 {
                b_width = 1;
            };
            self.branches.push(BonsaiBranch::new(
                last_trunk_step.pos,
                dir,
                b_width as usize,
                BranchShape::default_branch(),
                BROWN,
                self.appearance.leaf_count,
                self.appearance.leaf_type,
                self.appearance.leaf_color)
            );
        }
        //println!("{}", self.branches.len());
        //self.flush();
        let content = self.stdout.clone().into_inner();
        //let string = String::from_utf8(content).unwrap();
        
        std::io::stdout().write_all(&content).unwrap();
    }


    /// Helper function to flush the screen
    pub fn flush(&mut self) {
        self.stdout.flush().unwrap();
    }
}



/// Helper function to draw anything on the screen at a specified position
pub fn draw(stdout: &mut Writer, pos: (u16, u16), what: &str, color: Color) {
    //return;
    queue!(stdout,
        cursor::MoveTo(pos.0.into(), pos.1.into()),
        style::PrintStyledContent(what.with(color))
    ).unwrap();
}