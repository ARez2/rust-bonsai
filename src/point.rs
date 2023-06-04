use std::{fmt, ops::Add};


/// Helpers struct to represent a 2D point
#[derive(Clone, Copy)]
pub struct Point<T: PartialEq + Ord> {
    pub x: T,
    pub y: T,
}

impl<T: Ord + std::fmt::Display> fmt::Display for Point<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}

impl<T: Ord + std::fmt::Display + PartialEq> From<(T, T)> for Point<T> {
    fn from((x, y): (T, T)) -> Point<T> {
        Point {
            x,
            y,
        }
    }
}

impl<T: Add<Output = T> + Ord> Add for Point<T> {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}