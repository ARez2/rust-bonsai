use crossterm::style::Color;
use rand::{seq::SliceRandom, Rng};


use crate::{point::Point, RNG, ROSE};


/// Types of bases
#[derive(Clone, Copy)]
pub enum BaseType {
    LargePot,
    SmallPot,
}


/// Type of leaves. See get_leaf_string()
#[derive(Clone, Copy)]
pub enum LeafType {
    Pointy,
    Round,
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


/// Defines the appearance of a bonsai tree.
pub struct TreeAppearance {
    /// How far the leaves extend from the back of a branch
    pub leaf_count: u8,
    /// Which character to use for the leaf
    pub leaf_type: LeafType,
    pub leaf_color: Color,
    /// The random extents of the leaves in x dir (x = min, y = max)
    pub leafshape_x: Point<i16>,
    // The random extents of the leaves in y dir (x = min, y = max)
    pub leafshape_y: Point<i16>,
    /// Starting trunk width
    pub trunk_width: u8,
    /// Shape of the trunk (i.e. width falloff)
    pub trunk_shape: BranchShape,
    /// Shape of any branch (i.e. width falloff)
    pub branch_shape: BranchShape,
    /// Bonus for trunk width
    pub trunk_width_bonus: i16,
    /// Type of base
    pub base: BaseType
}

impl TreeAppearance {
    pub fn randomize(rng: &mut RNG, trunk_width: u8) -> TreeAppearance {
        // Extra leaf size based on trunk width
        let trunk_width_bonus = (trunk_width as f32 / 5.0 as f32).round() as i16;

        let leaf_type = [
            LeafType::Pointy,
            LeafType::Round,
            ].choose(rng).unwrap().clone();
        let (leafshape_x, leafshape_y) = match leaf_type {
            LeafType::Pointy => {
                let sidewards = rng.gen_range(1..=3) + trunk_width_bonus;
                (Point::from((sidewards, sidewards)), Point::from((1 + trunk_width_bonus, 0)))
            },
            LeafType::Round => {
                let sidewards = rng.gen_range(1..=3) + trunk_width_bonus;
                (Point::from((sidewards, sidewards)), Point::from((rng.gen_range(1..=3), rng.gen_range(2..=(3+trunk_width_bonus)))))
            }
        };
        
        let color_arr = match rng.gen_bool(0.05) {
            true => vec![Color::Rgb { r: 1, g: 1, b: 1 }],
            false => vec![Color::Green, Color::Red, Color::Yellow, ROSE],
        };

        TreeAppearance {
            trunk_shape: BranchShape {
                width_loose_chance: 1.0,
                min_width_loose_chance: 0.23,
                width_loose_ratio: 0.8
            },
            branch_shape: BranchShape {
                width_loose_chance: 0.3,
                min_width_loose_chance: 0.28,
                width_loose_ratio: 0.8
            },
            
            leaf_count: rng.gen_range(1..=3),
            leaf_type,
            leaf_color: *color_arr.choose(rng).unwrap(),
            leafshape_x,
            leafshape_y,
            trunk_width,
            trunk_width_bonus,
            base: *[BaseType::LargePot, BaseType::SmallPot].choose(rng).unwrap(),
        }
    }


    /// Depending on the leaf type, returns one of the leaf appearance characters
    pub fn get_leaf_string(&self, rng: &mut RNG) -> String {
        match self.leaf_type {
            LeafType::Pointy => ["V", "W", "VW", "WVW"].choose(rng).unwrap().to_string(),
            LeafType::Round => ["&", "o", "0"].choose(rng).unwrap().to_string(),
        }
    }
}