use crate::geometry::{Ray, Vec3, Vec3Ext};

const WORLD_UP: Vec3 = Vec3::new(0., 1., 0.);

#[derive(Clone, Copy)]
pub struct Camera {
    position: Vec3,
    yaw: f32,
    pitch: f32,
    fov: f32,
}

#[derive(Clone, Copy)]
pub struct CameraFrame {
    origin: Vec3,
    forward: Vec3,
    right: Vec3,
    up: Vec3,
    view: f32,
    aspect: f32,
}

impl Camera {
    /// Starts at the old orbit view, then behaves as a free-flying spectator camera.
    pub fn from_orbit(
        target: Vec3,
        orbit_yaw: f32,
        orbit_pitch: f32,
        distance: f32,
        fov: f32,
    ) -> Self {
        let yaw = orbit_yaw.to_radians();
        let pitch = orbit_pitch.to_radians();
        let position = target
            + Vec3::new(
                pitch.cos() * yaw.cos(),
                pitch.sin(),
                pitch.cos() * yaw.sin(),
            ) * distance;

        Self {
            position,
            // The old angles described the camera position around the target;
            // free-flight angles describe the direction in which it looks.
            yaw: orbit_yaw + 180.,
            pitch: -orbit_pitch,
            fov,
        }
    }

    pub fn frame(&self, width: u32, height: u32) -> CameraFrame {
        let forward = self.forward();
        let right = forward.cross(&WORLD_UP).unit();
        CameraFrame {
            origin: self.position,
            forward,
            right,
            up: right.cross(&forward).unit(),
            view: (self.fov.to_radians() * 0.5).tan(),
            aspect: width as f32 / height as f32,
        }
    }

    pub fn translate(&mut self, forward_amount: f32, right_amount: f32, up_amount: f32) {
        let forward = self.forward();
        let right = forward.cross(&WORLD_UP).unit();
        self.position += forward * forward_amount + right * right_amount + WORLD_UP * up_amount;
    }

    pub fn rotate(&mut self, yaw_amount: f32, pitch_amount: f32) {
        self.yaw += yaw_amount;
        self.pitch = (self.pitch + pitch_amount).clamp(-89., 89.);
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn yaw(&self) -> f32 {
        self.yaw
    }

    fn forward(&self) -> Vec3 {
        let yaw = self.yaw.to_radians();
        let pitch = self.pitch.to_radians();
        Vec3::new(
            pitch.cos() * yaw.cos(),
            pitch.sin(),
            pitch.cos() * yaw.sin(),
        )
        .unit()
    }
}

impl CameraFrame {
    pub fn ray(self, x: u32, y: u32, width: u32, height: u32) -> Ray {
        self.ray_sample(x, y, width, height, 0.5, 0.5)
    }

    /// Creates a ray through an arbitrary sub-pixel position. This allows the
    /// renderer to combine several rays and smooth diagonal edges cleanly.
    pub fn ray_sample(
        self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        offset_x: f32,
        offset_y: f32,
    ) -> Ray {
        let sx = (((x as f32 + offset_x) / width as f32) * 2. - 1.) * self.aspect * self.view;
        let sy = (1. - ((y as f32 + offset_y) / height as f32) * 2.) * self.view;
        Ray::new(
            self.origin,
            (self.forward + self.right * sx + self.up * sy).unit(),
        )
    }
}
