mod canvas;
mod sphere;
mod viewport;
mod world;
mod light; 
mod material;
mod camera;
mod util;
use nalgebra::{Vector2, Vector3};

pub use canvas::*;
pub use viewport::*;
pub use sphere::*;
pub use world::*;
pub use light::*;
pub use material::*;
pub use camera::*;

pub type CanvasVector = Vector2<usize>;
pub type WorldVector = Vector3<f64>;
pub type Colour = Vector3<u8>;

pub struct Ray {
    from: WorldVector,
    to: WorldVector,
}

impl Ray {
    pub fn new(from: WorldVector, to: WorldVector) -> Self {
        Self { from, to }
    }
    #[inline]
    pub fn point(&self, t: f64) -> WorldVector {
        self.from - t * self.direction()
    }
    #[inline]
    pub fn direction(&self) -> WorldVector {
        self.from - self.to
    }
}
