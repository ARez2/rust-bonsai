use crossterm::style::Color;
use rand::{seq::SliceRandom, Rng};


use crate::{point::Point, RNG, ROSE, branch::LeafType};


/// Types of bases
#[derive(Clone, Copy)]
pub enum BaseType {
    LargePot,
    SmallPot,
}



/// Defines the appearance of a bonsai tree.
pub struct TreeAppearance {
    /// How far the leaves extend from the back of a branch
    pub leaf_count: usize,
    /// Which character to use for the leaf
    pub leaf_type: LeafType,
    pub leaf_color: Color,
    /// Starting trunk width
    pub trunk_width: usize,
    /// Bonus for trunk width
    pub trunk_width_bonus: i16,
    /// Type of base
    pub base: BaseType
}

impl TreeAppearance {
    pub fn randomize(rng: &mut RNG, trunk_width: usize) -> TreeAppearance {
        // Extra leaf size based on trunk width
        let trunk_width_bonus = (trunk_width as f32 / 5.0 as f32).round() as i16;

        let leaf_type = [
            LeafType::Pointy,
            LeafType::Round,
            ].choose(rng).unwrap().clone();
        
        let color_arr = match rng.gen_bool(0.05) {
            true => vec![Color::Rgb { r: 1, g: 1, b: 1 }],
            false => vec![Color::Green, Color::Red, Color::Yellow, ROSE],
        };

        TreeAppearance {
            leaf_count: rng.gen_range(2..=4),
            leaf_type,
            leaf_color: *color_arr.choose(rng).unwrap(),
            trunk_width,
            trunk_width_bonus,
            base: *[BaseType::LargePot, BaseType::SmallPot].choose(rng).unwrap(),
        }
    }

    pub fn get_base(&self, margin: usize) -> String {
        let mut base = String::from(" ");
        match self.base {
            BaseType::LargePot => {
                base.push_str(&"_".repeat(margin));
                base.push_str(&" ".repeat(self.trunk_width));
                base.push_str(&"_".repeat(margin));
                base.push_str("\n");

                base.push_str("\\");
                base.push_str(&" ".repeat(margin - 2 + self.trunk_width));
                base.push_str("/\n");

                base.push_str(" \\");
                base.push_str(&" ".repeat(margin - 4 + self.trunk_width));
                base.push_str("/\n");
            },
            BaseType::SmallPot => {
                base.push_str(&"_".repeat(margin));
                base.push_str(&" ".repeat(self.trunk_width));
                base.push_str(&"_".repeat(margin));
                base.push_str("\n");

                base.push_str("(");
                base.push_str(&" ".repeat(margin - 2 + self.trunk_width));
                base.push_str(")\n");
            }
        };
        base
    }
}