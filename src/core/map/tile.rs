use bevy::prelude::Component;

#[derive(Component, Clone)]
pub struct Tile {
    pub index: usize,
    pub rotation: i32,
    pub bitmap: u16,
}

impl Tile {
    pub const SIZE: f32 = 32.; // Pixel size

    pub fn new(index: usize) -> Self {
        Self {
            index,
            rotation: 0,
            bitmap: match index {
                _ => 0b0000_0000_0000_0000,
            },
        }
    }

    pub fn from_rotation(index: usize, rotation: &i32) -> Self {
        Self::new(index).rotate(rotation).clone()
    }

    pub fn rotate(&mut self, rotation: &i32) -> &mut Self {
        self.rotation = *rotation;
        self.bitmap = match self.rotation {
            0 => self.bitmap,
            90 => (0..16).fold(0, |acc, i| {
                acc | (((self.bitmap >> i) & 1) << ((3 - i / 4) + (i % 4) * 4))
            }),
            180 => (0..16).fold(0, |acc, i| acc | (((self.bitmap >> i) & 1) << (15 - i))),
            270 => (0..16).fold(0, |acc, i| {
                acc | (((self.bitmap >> i) & 1) << ((i / 4) + (3 - i % 4) * 4))
            }),
            _ => panic!("Invalid rotation angle"),
        };
        self
    }
}
