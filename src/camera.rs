use crate::geometry::{Ray, Vec3, Vec3Ext};

#[derive(Clone, Copy)]
pub struct Camera {
    target: Vec3,
    yaw: f32,
    pitch: f32,
    distance: f32,
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
    pub fn new(target: Vec3, yaw: f32, pitch: f32, distance: f32, fov: f32) -> Self {
        Self {
            target,
            yaw,
            pitch,
            distance,
            fov,
        }
    }

    pub fn frame(&self, width: u32, height: u32) -> CameraFrame {
        let yaw = self.yaw.to_radians();
        let pitch = self.pitch.to_radians();
        let origin = self.target
            + Vec3::new(
                pitch.cos() * yaw.cos(),
                pitch.sin(),
                pitch.cos() * yaw.sin(),
            ) * self.distance;
        let forward = (self.target - origin).unit();
        let right = forward.cross(&Vec3::new(0., 1., 0.)).unit();
        CameraFrame {
            origin,
            forward,
            right,
            up: right.cross(&forward).unit(),
            view: (self.fov.to_radians() * 0.5).tan(),
            aspect: width as f32 / height as f32,
        }
    }

    pub fn rotate(&mut self, amount: f32) {
        self.yaw += amount;
    }
    pub fn tilt(&mut self, amount: f32) {
        self.pitch = (self.pitch + amount).clamp(-5., 75.);
    }
    pub fn zoom(&mut self, amount: f32) {
        self.distance = (self.distance + amount).clamp(8., 60.);
    }
    pub fn yaw(&self) -> f32 {
        self.yaw
    }
    pub fn distance(&self) -> f32 {
        self.distance
    }
}

impl CameraFrame {
    pub fn ray(self, x: u32, y: u32, width: u32, height: u32) -> Ray {
        let sx = (((x as f32 + 0.5) / width as f32) * 2. - 1.) * self.aspect * self.view;
        let sy = (1. - ((y as f32 + 0.5) / height as f32) * 2.) * self.view;
        Ray {
            origin: self.origin,
            direction: (self.forward + self.right * sx + self.up * sy).unit(),
        }
    }
}
