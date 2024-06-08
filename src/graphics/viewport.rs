// use nalgebra::Vector3;

// use super::{Canvas, CanvasVector, World, WorldVector};
// pub struct Viewport(pub WorldVector);

// impl Viewport {
//     pub fn from_aspect_ratio(aspect_ratio: f64) -> Self{
//         Self(WorldVector::new(1.0*aspect_ratio, 1.0, 1.0))
//     }
//     pub fn from_canvas_position(
//         &self,
//         canvas_position: CanvasVector,
//         canvas_size: CanvasVector,
//     ) -> WorldVector {
//         let xy = self
//             .0
//             .xy()
//             .component_div(&canvas_size.cast::<f64>())
//             .component_mul(&canvas_position.cast::<f64>())
//             - self.0.xy() / 2f64;
//         WorldVector::new(xy.x, -xy.y, self.0.z)
            
//     }
// }

// #[test]
// fn viewport_test() {
//     let v = Viewport::from_aspect_ratio(16.0/9.0);
//     println!("{:#?}", v.from_canvas_position(CanvasVector::new(8, 4), CanvasVector::new(16, 9)))
// }