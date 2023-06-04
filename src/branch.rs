use crate::{point::Point, RNG};
use rand::{Rng};
use simple_simplex::NoiseConfig;


/// Direction of a branch
#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Up,
    Left,
    Right,
}


/// Defines a step the branch has taken. This is used for drawing
#[derive(Clone)]
pub struct BonsaiStep {
    /// Current position of the step
    pub pos: Point<i16>,
    /// Difference to the last step (used for directional symbols)
    pub diff: (i16, i16),
    /// The branch width at this step
    pub width: u8,
}


pub struct BonsaiBranch {
    pub steps: Vec<BonsaiStep>,
    pub direction: Direction,
}

impl BonsaiBranch {
    /// Creates a new bonsai branch
    pub fn new(start_pos: Point<i16>, direction: Direction, start_width: u8) -> BonsaiBranch {
        BonsaiBranch {
            steps: vec![
                BonsaiStep {
                    pos: start_pos,
                    width: start_width,
                    diff: (0, 0),
                },
            ],
            direction,
        }
    }


    /// Handles a single step of a branch. Uses the last step and given arguments
    /// to define the next BonsaiStep's width and position
    /// returns if it was able to grow
    pub fn step(&mut self, noise: &NoiseConfig, rng: &mut RNG, mut chance_to_loose_width: f32, height_ratio: f32) -> bool {
        let last_step = self.steps.last().unwrap();
        if last_step.width < 1 {
            return false;
        };
        let mut new_width = last_step.width;
        
        // Force a certain width to ensure the tree ends at the terminal height
        let ratio = (height_ratio / 0.25).round() * 0.25;
        if ratio >= 0.75 && new_width >= 2 {
            new_width -= 1;
            chance_to_loose_width = 1.0;
        } else if ratio >= 0.5 && new_width > 4 {
            chance_to_loose_width = 0.75;
        } else if ratio >= 0.25 && new_width > 8 {
            chance_to_loose_width = 0.5;
        };

        if rng.gen_range(0.0..1.0) < chance_to_loose_width {
            new_width -= 1;
        };

        if self.steps.len() > 10 && self.direction != Direction::Up {
            new_width = 0;
        };


        let mut new_diff = (0 as i16, 0 as i16);
        let noise_val = noise.generate_range(last_step.pos.x.into(), last_step.pos.y.into()).round();
        //println!("{} {}", noise_val, noise_val.round());
        match self.direction {
            Direction::Up => {
                new_diff.0 += noise_val as i16;
                new_diff.1 -= 1;
                //if noise_val.abs() <= 1.0 {
                //};
            },
            Direction::Left => {
                new_diff.0 -= noise_val.abs() as i16;
                new_diff.0 -= new_width as i16;
                if self.steps.len() > 3 && rng.gen_bool(0.3) {
                    new_diff.1 -= 1 as i16;
                };
            },
            Direction::Right => {
                new_diff.0 += noise_val.abs() as i16;
                new_diff.0 += new_width as i16;
                if self.steps.len() > 3 && rng.gen_bool(0.3) {
                    new_diff.1 -= 1 as i16;
                };
            }
        };

        let mut new_pos = last_step.pos + Point::<i16>::from(new_diff);
        // The space it leaves for the input line of the terminal (where you type commands)
        let margin = 3;
        new_pos.x = std::cmp::max(new_pos.x, margin);
        new_pos.y = std::cmp::max(new_pos.y, margin);
        
        let new_step = BonsaiStep {
            pos: new_pos,
            width: new_width,
            diff: new_diff,
        };

        self.steps.push(new_step);
        true
    }
}