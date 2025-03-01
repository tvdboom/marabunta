/// Rotate a bitmap by `rotation` degrees
pub fn rotate_bitmap(bitmap: u16, rotation: i32) -> u16 {
    match rotation {
        0 => bitmap,
        90 => (0..16).fold(0, |acc, i| {
            acc | (((bitmap >> i) & 1) << ((3 - i / 4) + (i % 4) * 4))
        }),
        180 => (0..16).fold(0, |acc, i| acc | (((bitmap >> i) & 1) << (15 - i))),
        270 => (0..16).fold(0, |acc, i| {
            acc | (((bitmap >> i) & 1) << ((i / 4) + (3 - i % 4) * 4))
        }),
        _ => panic!("Invalid rotation angle"),
    }
}
