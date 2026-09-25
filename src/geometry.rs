use nalgebra::Vector3;

pub type Vec3 = Vector3<f32>;
pub const EPSILON: f32 = 0.002;

pub trait Vec3Ext {
    fn unit(self) -> Self;
    fn hadamard(self, other: Self) -> Self;
    fn clamp_rgb(self) -> Self;
}

impl Vec3Ext for Vec3 {
    fn unit(self) -> Self {
        let length = self.norm();
        if length > 0.0 {
            self / length
        } else {
            self
        }
    }

    fn hadamard(self, other: Self) -> Self {
        self.component_mul(&other)
    }

    fn clamp_rgb(self) -> Self {
        Self::new(
            self.x.clamp(0.0, 1.0),
            self.y.clamp(0.0, 1.0),
            self.z.clamp(0.0, 1.0),
        )
    }
}

#[derive(Clone, Copy)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    pub fn at(self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}

#[derive(Clone, Copy)]
pub struct Cube {
    pub min: Vec3,
    pub max: Vec3,
    pub material: usize,
}

#[derive(Clone, Copy)]
pub struct Hit {
    pub t: f32,
    pub point: Vec3,
    pub normal: Vec3,
    pub material: usize,
    pub uv: (f32, f32),
}

#[derive(Clone, Copy)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    pub fn union(a: Self, b: Self) -> Self {
        Self {
            min: Vec3::new(
                a.min.x.min(b.min.x),
                a.min.y.min(b.min.y),
                a.min.z.min(b.min.z),
            ),
            max: Vec3::new(
                a.max.x.max(b.max.x),
                a.max.y.max(b.max.y),
                a.max.z.max(b.max.z),
            ),
        }
    }

    pub fn entry(self, ray: Ray, max_t: f32) -> Option<f32> {
        let inv = Vec3::new(
            1.0 / ray.direction.x,
            1.0 / ray.direction.y,
            1.0 / ray.direction.z,
        );
        let x1 = (self.min.x - ray.origin.x) * inv.x;
        let x2 = (self.max.x - ray.origin.x) * inv.x;
        let y1 = (self.min.y - ray.origin.y) * inv.y;
        let y2 = (self.max.y - ray.origin.y) * inv.y;
        let z1 = (self.min.z - ray.origin.z) * inv.z;
        let z2 = (self.max.z - ray.origin.z) * inv.z;
        let near = x1.min(x2).max(y1.min(y2)).max(z1.min(z2));
        let far = x1.max(x2).min(y1.max(y2)).min(z1.max(z2));

        if far >= near && far >= EPSILON && near <= max_t {
            Some(near.max(EPSILON))
        } else {
            None
        }
    }
}

impl Cube {
    pub fn bounds(self) -> Aabb {
        Aabb {
            min: self.min,
            max: self.max,
        }
    }

    pub fn intersect(self, ray: Ray, t_max: f32) -> Option<Hit> {
        let inv = Vec3::new(
            1.0 / ray.direction.x,
            1.0 / ray.direction.y,
            1.0 / ray.direction.z,
        );
        let x1 = (self.min.x - ray.origin.x) * inv.x;
        let x2 = (self.max.x - ray.origin.x) * inv.x;
        let y1 = (self.min.y - ray.origin.y) * inv.y;
        let y2 = (self.max.y - ray.origin.y) * inv.y;
        let z1 = (self.min.z - ray.origin.z) * inv.z;
        let z2 = (self.max.z - ray.origin.z) * inv.z;
        let near = x1.min(x2).max(y1.min(y2)).max(z1.min(z2));
        let far = x1.max(x2).min(y1.max(y2)).min(z1.max(z2));
        if far < near || far < EPSILON || near > t_max {
            return None;
        }

        let t = if near > EPSILON { near } else { far };
        let point = ray.at(t);
        let edge = 0.003;
        let (normal, uv) = if (point.x - self.min.x).abs() < edge {
            (Vec3::new(-1., 0., 0.), (point.z, point.y))
        } else if (point.x - self.max.x).abs() < edge {
            (Vec3::new(1., 0., 0.), (point.z, point.y))
        } else if (point.y - self.min.y).abs() < edge {
            (Vec3::new(0., -1., 0.), (point.x, point.z))
        } else if (point.y - self.max.y).abs() < edge {
            (Vec3::new(0., 1., 0.), (point.x, point.z))
        } else if (point.z - self.min.z).abs() < edge {
            (Vec3::new(0., 0., -1.), (point.x, point.y))
        } else {
            (Vec3::new(0., 0., 1.), (point.x, point.y))
        };

        Some(Hit {
            t,
            point,
            normal,
            material: self.material,
            uv,
        })
    }
}
