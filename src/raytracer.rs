use crate::geometry::{Ray, Vec3, Vec3Ext, EPSILON};
use crate::materials::sample_texture;
use crate::scene::Scene;

const MAX_BOUNCES: u32 = 3;
const SUN_DIRECTION: Vec3 = Vec3::new(-0.45, 0.72, -0.52);
const MOON_DIRECTION: Vec3 = Vec3::new(-0.38, 0.78, -0.50);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeOfDay {
    Day,
    Night,
}

impl TimeOfDay {
    pub fn label(self) -> &'static str {
        match self {
            Self::Day => "dia",
            Self::Night => "noche",
        }
    }

    pub fn toggled(self) -> Self {
        match self {
            Self::Day => Self::Night,
            Self::Night => Self::Day,
        }
    }
}

#[derive(Clone, Copy)]
struct Environment {
    light_direction: Vec3,
    light_color: Vec3,
    ambient_color: Vec3,
    direct_strength: f32,
    shadow_strength: f32,
}

impl Environment {
    fn for_time(time: TimeOfDay) -> Self {
        match time {
            TimeOfDay::Day => Self {
                light_direction: SUN_DIRECTION.normalize(),
                light_color: Vec3::new(1.0, 0.91, 0.75),
                ambient_color: Vec3::new(0.18, 0.22, 0.28),
                direct_strength: 0.84,
                shadow_strength: 0.14,
            },
            TimeOfDay::Night => Self {
                light_direction: MOON_DIRECTION.normalize(),
                light_color: Vec3::new(0.48, 0.62, 1.0),
                ambient_color: Vec3::new(0.025, 0.04, 0.09),
                direct_strength: 0.30,
                shadow_strength: 0.20,
            },
        }
    }
}

pub struct RayTracer {
    time_of_day: TimeOfDay,
    space_background: bool,
}

impl Default for RayTracer {
    fn default() -> Self {
        Self {
            time_of_day: TimeOfDay::Day,
            space_background: false,
        }
    }
}

impl RayTracer {
    pub fn set_space_background(&mut self) {
        self.space_background = true;
        self.time_of_day = TimeOfDay::Day;
    }

    fn background(&self, direction: Vec3) -> Vec3 {
        if self.space_background {
            // Cubemap-like star cells: no sun, moon, disks or circular halos.
            let axis = direction.abs().imax();
            let u = direction[(axis + 1) % 3] / direction[axis].abs() * 110.;
            let v = direction[(axis + 2) % 3] / direction[axis].abs() * 110.;
            let seed = hash(u.floor(), v.floor());
            if seed > 0.987 && fract(u) < 0.16 && fract(v) < 0.16 {
                return Vec3::new(0.55, 0.72, 1.);
            }
            return Vec3::new(0.004, 0.007, 0.020);
        }
        skybox(direction, self.time_of_day)
    }

    pub fn set_time_of_day(&mut self, time_of_day: TimeOfDay) {
        self.time_of_day = time_of_day;
    }

    pub fn time_of_day(&self) -> TimeOfDay {
        self.time_of_day
    }

    pub fn trace(&self, scene: &Scene, ray: Ray) -> Vec3 {
        self.trace_recursive(scene, ray, 0)
    }

    fn trace_recursive(&self, scene: &Scene, ray: Ray, depth: u32) -> Vec3 {
        if depth >= MAX_BOUNCES {
            return self.background(ray.direction);
        }
        let Some(hit) = scene.hit(ray, f32::INFINITY) else {
            return self.background(ray.direction);
        };

        let environment = Environment::for_time(self.time_of_day);
        let material = scene.material(hit.material);
        let albedo = sample_texture(material.texture, hit.uv.0, hit.uv.1).hadamard(material.albedo);
        let shadow = scene.occluded(Ray::new(
            hit.point + hit.normal * EPSILON,
            environment.light_direction,
        ));
        let visibility = if shadow {
            environment.shadow_strength
        } else {
            1.0
        };
        let diffuse = hit.normal.dot(&environment.light_direction).max(0.)
            * visibility
            * environment.direct_strength;
        let halfway = (environment.light_direction - ray.direction).unit();
        let specular = hit.normal.dot(&halfway).max(0.).powf(material.shininess)
            * material.specular
            * if shadow { 0. } else { 1. };
        let mut local = albedo.hadamard(environment.ambient_color)
            + albedo.hadamard(environment.light_color) * diffuse
            + environment.light_color * specular;
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

fn smooth_noise(x: f32, y: f32) -> f32 {
    let ix = x.floor();
    let iy = y.floor();
    let fx = fract(x);
    let fy = fract(y);
    let ux = fx * fx * (3. - 2. * fx);
    let uy = fy * fy * (3. - 2. * fy);
    let bottom = hash(ix, iy) * (1. - ux) + hash(ix + 1., iy) * ux;
    let top = hash(ix, iy + 1.) * (1. - ux) + hash(ix + 1., iy + 1.) * ux;
    bottom * (1. - uy) + top * uy
}

fn cloud_density(direction: Vec3) -> f32 {
    if direction.y <= 0.015 {
        return 0.;
    }

    // Project the cloud layer onto a high horizontal plane. Several smooth
    // octaves avoid the old square cells while retaining a procedural sky.
    let scale = 0.75 / direction.y.max(0.09);
    let x = direction.x * scale;
    let z = direction.z * scale;
    let noise = smooth_noise(x * 2.2 + 8.1, z * 2.2 - 3.7) * 0.54
        + smooth_noise(x * 4.6 - 11.4, z * 4.6 + 7.2) * 0.29
        + smooth_noise(x * 9.2 + 2.8, z * 9.2 + 14.1) * 0.17;
    let horizon_fade = ((direction.y - 0.02) / 0.16).clamp(0., 1.);
    ((noise - 0.51) / 0.22).clamp(0., 1.) * horizon_fade
}

fn skybox(direction: Vec3, time: TimeOfDay) -> Vec3 {
    let height = (direction.y * 0.5 + 0.5).clamp(0., 1.);
    match time {
        TimeOfDay::Day => day_sky(direction, height),
        TimeOfDay::Night => night_sky(direction, height),
    }
}

fn day_sky(direction: Vec3, height: f32) -> Vec3 {
    let horizon = Vec3::new(0.72, 0.86, 1.0);
    let zenith = Vec3::new(0.075, 0.27, 0.68);
    let shaped_height = height.powf(0.65);
    let mut color = horizon * (1. - shaped_height) + zenith * shaped_height;

    let sun_direction = SUN_DIRECTION.normalize();
    let sun_dot = direction.dot(&sun_direction).max(0.);
    let sun_disk = sun_dot.powf(1500.);
    let sun_glow = sun_dot.powf(24.);
    color += Vec3::new(1.0, 0.64, 0.27) * sun_glow * 0.32;
    color += Vec3::new(1.0, 0.91, 0.70) * sun_disk * 4.0;

    let clouds = cloud_density(direction);
    if clouds > 0. {
        let rim = direction.dot(&sun_direction).max(0.).powf(5.);
        let cloud_color = Vec3::new(0.78, 0.84, 0.90) + Vec3::new(0.30, 0.20, 0.10) * rim;
        color = color * (1. - clouds * 0.72) + cloud_color * clouds * 0.88;
    }
    color
}

fn night_sky(direction: Vec3, height: f32) -> Vec3 {
    let horizon = Vec3::new(0.035, 0.055, 0.13);
    let zenith = Vec3::new(0.0025, 0.006, 0.025);
    let mut color = horizon * (1. - height) + zenith * height;

    if direction.y > 0. {
        color += stars(direction) * ((direction.y / 0.16).clamp(0., 1.));
    }

    let moon_direction = MOON_DIRECTION.normalize();
    let moon_dot = direction.dot(&moon_direction).clamp(0., 1.);
    let moon_disk = ((moon_dot - 0.9987) / 0.0013).clamp(0., 1.);
    let moon_glow = moon_dot.powf(180.);
    color += Vec3::new(0.38, 0.52, 1.0) * moon_glow * 0.55;
    color += Vec3::new(0.88, 0.92, 1.0) * moon_disk * 2.8;

    let clouds = cloud_density(direction) * 0.55;
    if clouds > 0. {
        let moon_lighting = moon_dot.powf(7.);
        let cloud_color =
            Vec3::new(0.07, 0.085, 0.15) + Vec3::new(0.18, 0.23, 0.42) * moon_lighting;
        color = color * (1. - clouds * 0.60) + cloud_color * clouds;
    }
    color
}

fn stars(direction: Vec3) -> Vec3 {
    let longitude = direction.z.atan2(direction.x) / std::f32::consts::TAU + 0.5;
    let latitude = direction.y.asin() / std::f32::consts::PI + 0.5;
    let cell_x = (longitude * 720.).floor();
    let cell_y = (latitude * 360.).floor();
    let seed = hash(cell_x, cell_y);
    if seed < 0.982 {
        return Vec3::zeros();
    }

    let local_x = fract(longitude * 720.) - 0.5;
    let local_y = fract(latitude * 360.) - 0.5;
    let core = (1. - (local_x * local_x + local_y * local_y).sqrt() * 5.).clamp(0., 1.);
    let brightness = core.powf(2.) * ((seed - 0.982) / 0.018).clamp(0.35, 1.0);
    let tint = if hash(cell_y + 41., cell_x - 17.) > 0.72 {
        Vec3::new(0.65, 0.78, 1.0)
    } else {
        Vec3::new(1.0, 0.94, 0.80)
    };
    tint * brightness * 1.8
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day_and_night_have_distinct_zenith_colors() {
        let up = Vec3::new(0., 1., 0.);
        let day = skybox(up, TimeOfDay::Day);
        let night = skybox(up, TimeOfDay::Night);
        assert!(day.norm() > night.norm() * 3.);
    }

    #[test]
    fn moon_is_bright_and_blue_white() {
        let moon = night_sky(MOON_DIRECTION.normalize(), 0.9);
        assert!(moon.x > 0.8);
        assert!(moon.z >= moon.x);
    }
}
