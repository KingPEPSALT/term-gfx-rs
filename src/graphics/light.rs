use nalgebra::Vector3;
use num_traits::Pow;

use super::WorldVector;

pub trait LightSource {
    fn diffuse(&self, point: WorldVector, normal: WorldVector) -> LightColour;
    fn specular(
        &self,
        viewing_direction: WorldVector,
        point: WorldVector,
        normal: WorldVector,
        specular_exponent: f64,
    ) -> LightColour;
    fn light_direction(&self, point: WorldVector) -> WorldVector;
    fn t_max(&self) -> f64;
}

pub type LightColour = Vector3<f64>;

/// Omnidirectional lightsource
#[derive(Clone, Copy)]
pub struct PointLight {
    pub position: WorldVector,
    pub colour: LightColour,
}
#[derive(Clone, Copy)]
pub struct DirectionalLight {
    pub direction: WorldVector,
    pub colour: LightColour,
}
impl LightSource for PointLight {
    fn diffuse(&self, point: WorldVector, normal: WorldVector) -> LightColour {
        let direction = point - self.position;
        self.colour.cast::<f64>()
            * (normal.dot(&direction) / (normal.magnitude() * direction.magnitude()))
    }

    fn specular(
        &self,
        viewing_direction: WorldVector,
        point: WorldVector,
        normal: WorldVector,
        specular_exponent: f64,
    ) -> LightColour {
        let direction = point - self.position;
        let r = 2.0 * normal.scale(normal.dot(&direction)) - direction;
        self.colour.scale(
            (r.dot(&viewing_direction) / (r.magnitude() * viewing_direction.magnitude()))
                .pow(specular_exponent),
        )
    }

    fn light_direction(&self, point: WorldVector) -> WorldVector {
        self.position - point
    }

    fn t_max(&self) -> f64 {
        1.0 
    }
}

impl LightSource for DirectionalLight {
    fn diffuse(&self, _: WorldVector, normal: WorldVector) -> LightColour {
        self.colour.cast::<f64>()
            * (normal.dot(&self.direction) / (normal.magnitude() * self.direction.magnitude()))
    }

    fn specular(
        &self,
        viewing_direction: WorldVector,
        _: WorldVector,
        normal: WorldVector,
        specular_exponent: f64,
    ) -> LightColour {
        let r = 2.0 * normal.scale(normal.dot(&self.direction)) - self.direction;
        self.colour.scale(
            (r.dot(&viewing_direction) / (r.magnitude() * viewing_direction.magnitude()))
                .pow(specular_exponent),
        )
    }

    fn light_direction(&self, _: WorldVector) -> WorldVector {
        self.direction
    }

    fn t_max(&self) -> f64 {
        f64::MAX
    }
}
