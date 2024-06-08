use nalgebra::{Matrix4, Vector2, Vector3, Vector4};

use super::{util::Direction, CanvasVector, WorldVector};

pub struct Camera {
    aspect_ratio: f64,
    fov: f64,
    
    pub position: WorldVector,

    pub basis: Basis,

    pub yaw: f64,
    pub pitch: f64,

    pub z_near: f64,
    pub z_far: f64,
    movement: Direction,
}

pub struct Basis {
    pub forward: WorldVector,
    pub right: WorldVector,
    pub up: WorldVector
}

const MODEL_MATRIX: Matrix4<f64> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0, 
    0.0, 0.0, 1.0, 0.0, 
    0.0, 0.0, 0.0, 1.0
); 

impl Camera {
    const MOVEMENT_SPEED: f64 = 1.0;
    const MOUSE_SENSITITVITY: f64 = 0.03;
    pub fn new(canvas_size: CanvasVector) -> Self {
        Self {

            aspect_ratio: canvas_size.x as f64/canvas_size.y as f64,
            fov: std::f64::consts::PI / 2.0, // radians, 90 degrees 

            position: Vector3::new(0.0, 0.0, 0.0),
            basis: Basis {
                forward: Vector3::z(),
                right: Vector3::x(),
                up: Vector3::y(),
            },

            yaw: 0.0,
            pitch: 0.0,

            z_near: 1_f64,
            z_far: 100000_f64,
            movement: Direction::empty(),
        }
    }
    pub fn get_perspective(&self) -> Matrix4<f64> {
        Matrix4::new_perspective(self.aspect_ratio, self.fov, self.z_near, self.z_far)
    }
    pub fn get_view(&self) -> Matrix4<f64> {
        Matrix4::look_at_rh(
            &self.position.into(),
            &(-self.basis.forward - self.position).into(),
            &self.basis.up,
        )
    }

    pub fn from_canvas(&self, canvas_position: CanvasVector, canvas_size: CanvasVector) -> WorldVector {
        (self.get_view() * Vector4::new(-(canvas_position.x as f64)/canvas_size.x as f64, canvas_position.y as f64/canvas_size.y as f64, self.z_near, 1.0)).xyz()
    }
    #[inline]
    pub fn model_view(&self) -> Matrix4<f64> {
        self.get_view() * MODEL_MATRIX
    }

    #[inline]
    pub fn projection(&self) -> Matrix4<f64> {
        self.get_perspective() * self.model_view()
    }

    pub fn get_movement_direction(&self) -> Vector3<f64> {
        let forward = self.basis.forward.component_mul(&WorldVector::new(1.0, 0.0, 1.0)).normalize();
        let movement_basis = Basis {
            forward,
            up: WorldVector::y(),
            right: forward.cross(&WorldVector::y())
        };
        let movement = 
            movement_basis.forward
                .scale(self.movement.contains(Direction::FORWARD).into())
            - movement_basis.forward
                .scale(self.movement.contains(Direction::BACKWARD).into())
            - movement_basis.right
                .scale(self.movement.contains(Direction::LEFT).into())
            + movement_basis.right
                .scale(self.movement.contains(Direction::RIGHT).into());

        return if movement == Vector3::zeros() {
            movement
        } else {
            movement.normalize()
                
        } + movement_basis.up.scale(self.movement.contains(Direction::UP).into())   // don't normalize up/down movement
        - movement_basis.up.scale(self.movement.contains(Direction::DOWN).into());
    }
    pub fn update(&mut self, delta: f64) {
        let movement = self.get_movement_direction();
        self.position += movement * Self::MOVEMENT_SPEED * delta/1000.0;
    }
    pub fn process_mouse_motion(&mut self, mouse_delta: Vector2<i16>, delta: f64) {
        self.yaw -= Self::MOUSE_SENSITITVITY * delta/1000.0 * mouse_delta.x as f64;
        self.pitch += (Self::MOUSE_SENSITITVITY * delta/1000.0 * mouse_delta.y as f64)
            .clamp(-std::f64::consts::FRAC_PI_2+0.01, std::f64::consts::FRAC_PI_2-0.01);

        let (cos_yaw, sin_yaw) = (self.yaw.cos(), -self.yaw.sin());
        let (cos_pitch, sin_pitch) = (self.pitch.cos(), self.pitch.sin());

        self.basis.forward =
            Vector3::new(sin_yaw * cos_pitch, sin_pitch, cos_yaw * cos_pitch).normalize();
        self.basis.right = Vector3::new(cos_yaw, 0.0, -sin_yaw).normalize();
        self.basis.up = self.basis.forward.cross(&self.basis.right).normalize();
    }
    pub fn input(&mut self) {
        self.movement.set(Direction::UP, inputbot::KeybdKey::SpaceKey.is_pressed()); // up
        self.movement.set(Direction::DOWN, inputbot::KeybdKey::LShiftKey.is_pressed()); // down
        self.movement.set(Direction::FORWARD, inputbot::KeybdKey::WKey.is_pressed()); // forward
        self.movement.set(Direction::BACKWARD, inputbot::KeybdKey::SKey.is_pressed()); // backwards
        self.movement.set(Direction::RIGHT, inputbot::KeybdKey::DKey.is_pressed()); // right
        self.movement.set(Direction::LEFT, inputbot::KeybdKey::AKey.is_pressed()); // left
    }
}
