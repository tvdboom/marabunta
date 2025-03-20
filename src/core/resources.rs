use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Resources {
    pub leaves: f32,
    pub nutrients: f32,
}

impl Resources {
    pub const MAX: Self = Self {
        leaves: f32::MAX,
        nutrients: f32::MAX,
    };

    pub fn new(leaves: f32, nutrients: f32) -> Self {
        Self { leaves, nutrients }
    }

    pub fn from_leaves(leaves: f32) -> Self {
        Self {
            leaves,
            nutrients: 0.,
        }
    }

    pub fn from_nutrients(nutrients: f32) -> Self {
        Self {
            leaves: 0.,
            nutrients,
        }
    }
}

impl PartialOrd for Resources {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let all_gte = self.leaves >= other.leaves && self.nutrients >= other.nutrients;

        let all_lte = self.leaves <= other.leaves && self.nutrients <= other.nutrients;

        match (all_gte, all_lte) {
            (true, true) => Some(Ordering::Equal),
            (true, false) => Some(Ordering::Greater),
            (false, true) => Some(Ordering::Less),
            (false, false) => None,
        }
    }
}

macro_rules! resources_binary_ops {
    ($($trait:ident, $method:ident, $op:tt);*;) => {
        $(
            // Binary operations with Resources reference
            impl $trait<&Self> for Resources {
                type Output = Self;

                fn $method(self, rhs: &Resources) -> Self::Output {
                    Self {
                        leaves: self.leaves $op rhs.leaves,
                        nutrients: self.nutrients $op rhs.nutrients,
                    }
                }
            }

            // Binary operations with float
            impl<T: Into<f32>> $trait<T> for Resources {
                type Output = Self;

                fn $method(self, rhs: T) -> Self::Output {
                    let float = rhs.into();
                    Self {
                        leaves: self.leaves $op float,
                        nutrients: self.nutrients $op float,
                    }
                }
            }

            // Binary operations with float on reference
            impl<T: Into<f32>> $trait<T> for &Resources {
                type Output = Resources;

                fn $method(self, rhs: T) -> Resources {
                    let float = rhs.into();
                    Resources {
                        leaves: self.leaves $op float,
                        nutrients: self.nutrients $op float,
                    }
                }
            }
        )*
    };
}

resources_binary_ops!(
    Add, add, +;
    Sub, sub, -;
    Mul, mul, *;
    Div, div, /;
);

macro_rules! resources_assignment_ops {
    ($($trait:ident, $method:ident, $op:tt);*;) => {
        $(
            // Assignment operations with Resources
            impl $trait<Self> for Resources {
                fn $method(&mut self, rhs: Self) {
                    self.leaves $op rhs.leaves;
                    self.nutrients $op rhs.nutrients;
                }
            }

            // Assignment operations with Resources reference
            impl $trait<&Self> for Resources {
                fn $method(&mut self, rhs: &Self) {
                    self.leaves $op rhs.leaves;
                    self.nutrients $op rhs.nutrients;
                }
            }

            // Assignment operations with float
            impl<T: Into<f32>> $trait<T> for Resources {
                fn $method(&mut self, rhs: T) {
                    let float = rhs.into();
                    self.leaves $op float;
                    self.nutrients $op float;
                }
            }
        )*
    };
}

resources_assignment_ops!(
    AddAssign, add_assign, +=;
    SubAssign, sub_assign, -=;
    MulAssign, mul_assign, *=;
    DivAssign, div_assign, /=;
);
