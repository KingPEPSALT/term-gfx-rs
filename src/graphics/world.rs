use nalgebra::clamp;
use num_traits::clamp_min;

use super::{Camera, Colour, LightColour, LightSource, Material, Ray, Sphere, WorldVector};

/// Defines a ray hit, what point it hit,
/// the normal of the point and what material
/// was hit
#[derive(Debug)]
pub struct Hit {
    pub point: WorldVector,
    pub normal: WorldVector,
    pub direction: WorldVector,
    pub material: Material,
}

/// Defines the lighting contribution on the
/// a spot, used for debug mostly
#[derive(Debug, Default)]
pub struct LightingContribution {
    pub ambient: LightColour,
    pub diffuse: LightColour,
    pub specular: LightColour,
}

impl LightingContribution {
    #[inline]
    pub fn total(&self) -> LightColour {
        clamp(
            self.diffuse + self.specular + self.ambient,
            LightColour::zeros(),
            LightColour::from_element(1.0),
        )
    }
}
/// Manages the objects within the world and
/// the light within the world
pub struct World {
    pub spheres: Vec<Sphere>,
    pub light_sources: Vec<Box<dyn LightSource>>,
    pub camera: Camera,
    pub ambient: LightColour,
}

impl World {
    pub fn closest_intersection(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<(usize, f64)> {
        let mut closest_t: f64 = f64::MAX;
        let mut closest_sphere_index: Option<usize> = None;
        for (i, sphere) in self.spheres.iter().enumerate() {
            match sphere.intersect(&ray) {
                Some((t1, t2)) => {
                    if t1 < t_max && t1 < closest_t && t1 > t_min {
                        closest_t = t1;
                        closest_sphere_index = Some(i);
                    }
                    if t2 < t_max && t2 < closest_t && t2 > t_min {
                        closest_t = t2;
                        closest_sphere_index = Some(i);
                    }
                }
                None => {}
            }
        }
        closest_sphere_index.map(|sphere_index| (sphere_index, closest_t))
    }

    pub fn trace_ray(&self, through: WorldVector, t_min: f64, t_max: f64) -> Option<Hit> {
        let ray = Ray::new(self.camera.position, through);
        let intersection = self.closest_intersection(&ray, t_min, t_max);

        match intersection {
            Some((sphere_index, t)) => {
                let sphere = &self.spheres[sphere_index];

                let point = ray.point(t);
                let normal = (sphere.center - point).normalize();

                Some(Hit {
                    point,
                    normal,
                    direction: -ray.direction(),
                    material: sphere.material.clone(),
                })
            }
            None => None,
        }
    }
    pub fn compute_lighting(&self, hit: &Hit) -> Colour {
        let lighting = self.get_lighting(hit).total();
        lighting
            .component_mul(&hit.material.colour)
            .scale(255.0)
            .try_cast::<u8>()
            .unwrap()
    }

    pub fn get_lighting(&self, hit: &Hit) -> LightingContribution {
        let mut lighting = LightingContribution {
            ambient: self.ambient,
            diffuse: LightColour::zeros(),
            specular: LightColour::zeros(),
        };
        for light_source in &self.light_sources {
            // diffuse lighting always applies
            lighting.diffuse += light_source.diffuse(hit.point, hit.normal);

            // shadows
            if self
                .closest_intersection(
                    &Ray::new(hit.point, light_source.light_direction(hit.point)),
                    0.001,
                    light_source.t_max(),
                )
                .is_some()
            {
                continue;
            }
            // apply specular if material has it
            if let Some(specular_exponent) = hit.material.specular {
                lighting.specular +=
                    light_source.specular(hit.direction, hit.point, hit.normal, specular_exponent)
            };
        }
        lighting.specular = clamp_min(lighting.specular, LightColour::zeros());
        lighting.diffuse = clamp_min(lighting.diffuse, LightColour::zeros());
        lighting
    }
}
