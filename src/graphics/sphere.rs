use super::{Material, Ray, WorldVector};
#[derive(Debug)]
pub struct Sphere {
    pub center: WorldVector,
    pub radius: f64,
    pub material: Material
}

impl Sphere {


    pub fn contains(&self, point: WorldVector) -> bool {
        let d = point - self.center;
        d.dot(&d) < self.radius * self.radius
    }

    pub fn intersect(&self, ray: &Ray) -> Option<(f64, f64)> {
        let dir = ray.direction();
        let a = dir.dot(&dir);
        let b = 2f64 * self.center.dot(&dir);
        let c = self.center.dot(&self.center) - self.radius*self.radius;
        let discriminant = b * b - 4f64 * a * c;
        if discriminant < 0f64 {
            return None;
        }
        Some((
            (-b + f64::sqrt(discriminant)) / (2f64 * a),
            (-b - f64::sqrt(discriminant)) / (2f64 * a),
        ))
    }
}
