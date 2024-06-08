use std::ops::{AddAssign, Neg};
use bitflags::bitflags;
use nalgebra::{Scalar, Vector3};
use num_traits::{One, Zero};

bitflags! {
    #[derive(PartialEq, Debug)]
    pub struct Direction: u8 {
        const BACKWARD = 1;
        const LEFT = 1 << 1;
        const DOWN = 1 << 2;
        const FORWARD = 1 << 3;
        const RIGHT = 1 << 4;
        const UP = 1 << 5;
    }
}

pub fn _direction_axes<T: Neg<Output = T> + Scalar + Zero + One>() -> [Vector3<T>; 6] {
    [
        -Vector3::z(),
        -Vector3::x(),
        -Vector3::y(),
        Vector3::z(),
        Vector3::x(),
        Vector3::y(),
    ]
}

pub fn _direction_vector<T: Neg<Output = T> + Scalar + Zero + One + AddAssign>(
    direction: Direction,
) -> Vector3<T> {
    let mut i = 0_u8;
    _direction_axes::<T>()
        .iter()
        .cloned()
        .reduce(|acc, direction_vector| {
            i += 1;
            acc + if direction.contains(Direction::from_bits(1 << i).unwrap()) {
                direction_vector
            } else {
                Vector3::zeros()
            }
        })
        .unwrap()
}
