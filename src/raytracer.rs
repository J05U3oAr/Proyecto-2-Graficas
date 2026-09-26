use crate::geometry::{Ray, Vec3, Vec3Ext, EPSILON};
use crate::materials::sample_texture;
use crate::scene::Scene;

const MAX_BOUNCES: u32 = 3;

pub struct RayTracer {
    sun_direction: Vec3,
}

impl Default for RayTracer {
    fn default() -> Self {
        Self {
            sun_direction: Vec3::new(-0.45, 0.72, -0.52).unit(),
        }
    }
}

impl RayTracer {
    pub fn trace(&self, scene: &Scene, ray: Ray) -> Vec3 {
        self.trace_recursive(scene, ray, 0)
    }

    fn trace_recursive(&self, scene: &Scene, ray: Ray, depth: u32) -> Vec3 {
        if depth >= MAX_BOUNCES {
            return skybox(ray.direction);
        }
        let Some(hit) = scene.hit(ray, f32::INFINITY) else {
            return skybox(ray.direction);
        };
        let material = scene.material(hit.material);
        let albedo = sample_texture(material.texture, hit.uv.0, hit.uv.1).hadamard(material.albedo);
        let shadow = scene.occluded(Ray::new(
            hit.point + hit.normal * EPSILON,
            self.sun_direction,
        ));
        let diffuse = hit.normal.dot(&self.sun_direction).max(0.) * if shadow { 0.14 } else { 1.0 };
        let halfway = (self.sun_direction - ray.direction).unit();
        let specular = hit.normal.dot(&halfway).max(0.).powf(material.shininess)
            * material.specular
            * if shadow { 0. } else { 1. };
        let mut local = albedo * (0.16 + diffuse * 0.84) + Vec3::new(1., 0.91, 0.75) * specular;
        let mut reflected = Vec3::zeros();

        if material.reflectivity > 0. || material.transparency > 0. {
            reflected = self.trace_recursive(
                scene,
                Ray::new(
                    hit.point + hit.normal * EPSILON,
                    reflect(ray.direction, hit.normal).unit(),
                ),
                depth + 1,
            );
        }
        if material.transparency > 0. {
            let transmitted = refract(ray.direction, hit.normal, material.ior)
                .map(|direction| {
                    self.trace_recursive(
                        scene,
                        Ray::new(hit.point + direction * EPSILON, direction),
                        depth + 1,
                    )
                })
                .unwrap_or(reflected);
            local = local * (1. - material.transparency)
                + transmitted.hadamard(albedo) * material.transparency;
        }
        local * (1. - material.reflectivity) + reflected * material.reflectivity
    }
}

fn fract(value: f32) -> f32 {
    value - value.floor()
}
fn hash(x: f32, y: f32) -> f32 {
    fract((x * 127.1 + y * 311.7).sin() * 43758.545)
}

fn skybox(direction: Vec3) -> Vec3 {
    let height = (direction.y * 0.5 + 0.5).clamp(0., 1.);
    let mut color =
        Vec3::new(0.76, 0.87, 0.98) * (1. - height) + Vec3::new(0.12, 0.36, 0.72) * height;
    if direction.y > 0.05 {
        let cloud = hash((direction.x * 18.).floor(), (direction.z * 18.).floor());
        if cloud > 0.78 {
            color += Vec3::new(0.18, 0.18, 0.16) * ((cloud - 0.78) / 0.22) * height;
        }
    }
    let sun = direction
        .dot(&Vec3::new(-0.45, 0.72, -0.52).unit())
        .max(0.)
        .powf(300.);
    color + Vec3::new(1., 0.78, 0.42) * sun * 3.
}

fn reflect(direction: Vec3, normal: Vec3) -> Vec3 {
    direction - normal * (2. * direction.dot(&normal))
}

fn refract(direction: Vec3, normal: Vec3, ior: f32) -> Option<Vec3> {
    let eta = if direction.dot(&normal) < 0. {
        1. / ior
    } else {
        ior
    };
    let normal = if direction.dot(&normal) < 0. {
        normal
    } else {
        -normal
    };
    let cos_i = (-direction).dot(&normal).clamp(0., 1.);
    let k = 1. - eta * eta * (1. - cos_i * cos_i);
    if k < 0. {
        None
    } else {
        Some((direction * eta + normal * (eta * cos_i - k.sqrt())).unit())
    }
}
