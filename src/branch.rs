use std::fmt::Write;

use crate::{point::Point, RNG, draw, Writer};
use crossterm::style::Color;
use rand::{Rng, seq::SliceRandom};
use simple_simplex::NoiseConfig;


/// Direction of a branch
#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Up,
    Left,
    Right,
}



/// Defines how a branch looks
#[derive(Clone, Copy)]
pub struct BranchShape {
    /// initial chance to loose width
    pub width_loose_chance: f32,
    /// Minimum chance to loose width
    pub min_width_loose_chance: f32,
    /// How much percent the width chance looses
    pub width_loose_ratio: f32,
}
impl BranchShape {
    pub fn default_trunk() -> Self {
        BranchShape {
            width_loose_chance: 1.0,
            min_width_loose_chance: 0.23,
            width_loose_ratio: 0.8
        }
    }

    pub fn default_branch() -> Self {
        BranchShape {
            width_loose_chance: 0.3,
            min_width_loose_chance: 0.28,
            width_loose_ratio: 0.8
        }
    }
}


const POINTY_LEAVES: [&'static str; 4] = ["V", "W", "VW", "WVW"];
const ROUND_LEAVES: [&'static str; 3] = ["&", "o", "0"];
/// Type of leaves. See get_leaf_string()
#[derive(Clone, Copy)]
pub enum LeafType {
    Pointy,
    Round,
} 

#[derive(Clone)]
pub struct Leaf {
    pub pos: Point<i16>,
    pub attached_to: Point<i16>,
    pub symbol: String,
    pub color: Color,
}
impl std::fmt::Debug for Leaf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("")
    }
}


/// Defines a step the branch has taken. This is used for drawing
#[derive(Debug, Clone)]
pub struct BonsaiStep {
    /// Current position of the step
    pub pos: Point<i16>,
    /// Difference to the last step (used for directional symbols)
    pub diff: (i16, i16),
    /// The branch width at this step
    pub width: usize,
}


pub struct BonsaiBranch {
    pub steps: Vec<BonsaiStep>,
    pub direction: Direction,
    pub shape: BranchShape,
    pub color: Color,
    pub base_leaf_color: Color,
    /// Each element represent one leaf attachment point with the max number of leaves and a vector of leaves
    pub leaves: Vec<(usize, Vec<Leaf>, (Point<i16>, Point<i16>))>,
    pub leaftype: LeafType,
    /// How many steps from the tip of the branch backwards there should be leaves
    max_leaf_positions: usize,
}

impl BonsaiBranch {
    /// Creates a new bonsai branch
    pub fn new(start_pos: Point<i16>, direction: Direction, start_width: usize, shape: BranchShape, color: Color, max_leaf_positions: usize, leaftype: LeafType, base_leaf_color: Color) -> BonsaiBranch {
        BonsaiBranch {
            steps: vec![
                BonsaiStep {
                    pos: start_pos,
                    width: start_width,
                    diff: (0, 0),
                },
            ],
            direction,
            shape,
            color,
            base_leaf_color,
            leaves: vec![],
            max_leaf_positions,
            leaftype,
        }
    }


    /// Handles a single step of a branch. Uses the last step and given arguments
    /// to define the next BonsaiStep's width and position
    /// returns if it was able to grow
    pub fn step(&mut self, noise: &NoiseConfig, rng: &mut RNG, screen_dimensions: (i16, i16), stdout: &mut Writer) -> bool {
        let last_step = self.steps.last().unwrap();
        if last_step.width < 1 {
            return false;
            if self.leaves.len() >= self.max_leaf_positions && {
                let last = self.leaves.last().unwrap();
                last.1.len() >= last.0
            } {
                return false;
            }
            if self.leaves.is_empty() {
                // Determine the attachment points of the leaves
                let mut positions: Vec<Point<i16>> = self.steps.iter().rev().map(|step| step.pos).collect();
                positions.truncate(self.max_leaf_positions);
                // Fill the leaves vector with relevant data
                positions.iter().for_each(|pos| {
                    // TODO: Adjust added width value
                    let num_leaves = rng.gen_range(5..=15+self.steps[0].width);
                    let (extents_min, extents_max) = match self.leaftype {
                        LeafType::Pointy => {
                            // Flat shape
                            (
                                Point::from((-rng.gen_range(3..=6), -rng.gen_range(0..=2))),
                                Point::from((rng.gen_range(3..=6), rng.gen_range(3..=6)))
                            )
                        },
                        LeafType::Round => {
                            // Bit more round shape
                            (
                                Point::from((-rng.gen_range(3..=6), -rng.gen_range(3..=6))),
                                Point::from((rng.gen_range(3..=6), rng.gen_range(2..=5)))
                            )
                        }
                    };
                    self.leaves.push((num_leaves, vec![], (extents_min, extents_max)));
                });
            }
            //self.grow_leaf(rng, stdout);
            return true;
        };

        let ratio = match self.direction {
            Direction::Up =>  1.0 - (last_step.pos.y as f32 / (screen_dimensions.1 - 1) as f32),
            Direction::Left =>  1.0 - (last_step.pos.x as f32 / (screen_dimensions.0 - 1) as f32),
            Direction::Right =>  last_step.pos.x as f32 / (screen_dimensions.0 - 1) as f32,
        };
        if self.shape.width_loose_chance > self.shape.min_width_loose_chance {
            self.shape.width_loose_chance *= self.shape.width_loose_ratio;
        } else {
            self.shape.width_loose_chance = self.shape.min_width_loose_chance;
        }
        self.shape.width_loose_chance = (self.shape.width_loose_chance * 10.0).round() / 10.0;
        
        let mut new_width = last_step.width;
        let mut chance_to_loose_width = self.shape.width_loose_chance;
        
        // HACK and ugly: Force a certain width to ensure the tree ends at the terminal height
        let ratio = (ratio / 0.25).round() * 0.25;
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

        let dir_string = self.get_string_for_dir(rng, new_step.diff, new_step.width);
        let mut draw_pos = new_step.pos;
        if self.direction != Direction::Up && new_step.width <= 1 {
            draw_pos.y += 1;
        }
        draw(stdout, (draw_pos.x as u16, draw_pos.y as u16), &dir_string, self.color);

        self.steps.push(new_step);
        true
    }


    fn grow_leaf(&mut self, rng: &mut RNG, stdout: &mut Writer) {
        let num_steps = self.steps.len() - 1;
        let attachment_point = {
            let mut pt = None;
            for (idx, (max_leaves, leaves, extents)) in self.leaves.iter_mut().enumerate() {
                if leaves.len() < *max_leaves {
                    pt = Some((self.steps[num_steps - idx].pos, leaves, extents));
                    break;
                }
            }
            pt
        };
        
        if let Some(point) = attachment_point {
            let symbol = Self::get_leaf_string(self.leaftype, rng);
            let color = match self.base_leaf_color {
                Color::Rgb { r, g, b } => {
                    Color::Rgb {
                        r: r.saturating_sub(rng.gen_range(0..20)),
                        g: g.saturating_sub(rng.gen_range(0..20)),
                        b: b.saturating_sub(rng.gen_range(0..20)),
                    }
                },
                _ => self.base_leaf_color,
            };
            let min = point.2.0;
            let max = point.2.1;
            let rand_x = rng.gen_range(min.x..=max.x);
            let rand_y = rng.gen_range(min.y..=max.y);
            let mut new_pos = point.0 + Point::from((rand_x, rand_y));
            new_pos.x = std::cmp::max(3, new_pos.x);
            new_pos.y = std::cmp::max(3, new_pos.y);
            
            //println!("Attach: {}, pos: {}", point.0, new_pos);
            
            point.1.push(Leaf {
                pos: new_pos,
                attached_to: point.0,
                symbol: symbol.clone(),
                color,
            });
            draw(stdout, (new_pos.x as u16, new_pos.y as u16), symbol.as_str(), color)
        }
    }

    /// Depending on the leaf type, returns one of the leaf appearance characters
    fn get_leaf_string(leaftype: LeafType, rng: &mut RNG) -> String {
        match leaftype {
            LeafType::Pointy => POINTY_LEAVES.choose(rng).unwrap().to_string(),
            LeafType::Round => ROUND_LEAVES.choose(rng).unwrap().to_string(),
        }
    }

    /// Depending on the direction, returns a string the looks like the direction
    pub fn get_string_for_dir(&mut self, rng: &mut RNG, mut dir: (i16, i16), width: usize) -> String {
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
            result.push(*set.choose(rng).unwrap_or(&'?'));
        };
        result
    }
}