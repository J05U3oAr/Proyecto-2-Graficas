use crate::geometry::{Ray, Vec3, Vec3Ext, EPSILON};
use crate::materials::sample_texture;
use crate::scene::Scene;

const MAX_BOUNCES: u32 = 3;
const DAY_PHASE: f32 = 0.25;
const NIGHT_PHASE: f32 = 0.75;

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
    fn for_cycle(cycle_time: f32) -> Self {
        let sun_direction = celestial_direction(cycle_time);
        let moon_direction = -sun_direction;
        let daylight = smoothstep(-0.08, 0.18, sun_direction.y);
        let moonlight = smoothstep(-0.08, 0.16, moon_direction.y) * (1. - daylight * 0.75);
        let twilight = twilight_strength(sun_direction.y);
        let dusk = dusk_amount(cycle_time);

        let warm_light = mix(Vec3::new(1.0, 0.72, 0.38), Vec3::new(1.0, 0.43, 0.22), dusk);
        let sunlight = mix(warm_light, Vec3::new(1.0, 0.93, 0.78), daylight);
        let moon_color = Vec3::new(0.44, 0.58, 1.0);
        let light_direction = if sun_direction.y > -0.035 {
            sun_direction
        } else {
            moon_direction
        };
        let light_color = if sun_direction.y > -0.035 {
            sunlight
        } else {
            moon_color
        };

        let night_ambient = Vec3::new(0.018, 0.03, 0.075);
        let day_ambient = Vec3::new(0.18, 0.22, 0.28);
        let twilight_ambient = mix(
            Vec3::new(0.22, 0.12, 0.15),
            Vec3::new(0.19, 0.075, 0.10),
            dusk,
        );
        let ambient_color = mix(night_ambient, day_ambient, daylight)
            + twilight_ambient * twilight * (1. - daylight * 0.55);

        Self {
            light_direction,
            light_color,
            ambient_color,
            direct_strength: if sun_direction.y > -0.035 {
                0.24 + daylight * 0.60 + twilight * 0.16
            } else {
                0.12 + moonlight * 0.20
            },
            shadow_strength: mix_scalar(0.22, 0.14, daylight),
        }
    }
}

pub struct RayTracer {
    cycle_time: f32,
    space_background: bool,
}

impl Default for RayTracer {
    fn default() -> Self {
        Self {
            cycle_time: DAY_PHASE,
            space_background: false,
        }
    }
}

impl RayTracer {
    pub fn set_space_background(&mut self) {
        self.space_background = true;
        self.cycle_time = DAY_PHASE;
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
        skybox(direction, self.cycle_time)
    }

    pub fn set_time_of_day(&mut self, time_of_day: TimeOfDay) {
        self.cycle_time = match time_of_day {
            TimeOfDay::Day => DAY_PHASE,
            TimeOfDay::Night => NIGHT_PHASE,
        };
    }

    pub fn time_of_day(&self) -> TimeOfDay {
        if celestial_direction(self.cycle_time).y >= 0. {
            TimeOfDay::Day
        } else {
            TimeOfDay::Night
        }
    }

    pub fn set_cycle_time(&mut self, cycle_time: f32) {
        self.cycle_time = cycle_time.rem_euclid(1.);
    }

    pub fn cycle_time(&self) -> f32 {
        self.cycle_time
    }

    pub fn cycle_label(&self) -> &'static str {
        let sun = celestial_direction(self.cycle_time);
        if sun.y > 0.20 {
            "dia"
        } else if sun.y >= -0.16 && dusk_amount(self.cycle_time) >= 0.5 {
            "atardecer"
        } else if sun.y >= -0.16 {
            "amanecer"
        } else {
            "noche"
        }
    }

    pub fn night_visibility(&self) -> f32 {
        let sun_height = celestial_direction(self.cycle_time).y;
        1. - smoothstep(-0.16, 0.06, sun_height)
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

        let environment = Environment::for_cycle(self.cycle_time);
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

fn celestial_direction(cycle_time: f32) -> Vec3 {
    let angle = cycle_time.rem_euclid(1.) * std::f32::consts::TAU;
    Vec3::new(-angle.cos() * 0.88, angle.sin(), -0.42).normalize()
}

fn dusk_amount(cycle_time: f32) -> f32 {
    let angle = cycle_time.rem_euclid(1.) * std::f32::consts::TAU;
    smoothstep(-0.35, 0.35, -angle.cos())
}

fn twilight_strength(sun_height: f32) -> f32 {
    1. - smoothstep(0.015, 0.42, sun_height.abs())
}

fn smoothstep(edge0: f32, edge1: f32, value: f32) -> f32 {
    let t = ((value - edge0) / (edge1 - edge0)).clamp(0., 1.);
    t * t * (3. - 2. * t)
}

fn mix(a: Vec3, b: Vec3, amount: f32) -> Vec3 {
    a * (1. - amount) + b * amount
}

fn mix_scalar(a: f32, b: f32, amount: f32) -> f32 {
    a * (1. - amount) + b * amount
}

fn skybox(direction: Vec3, cycle_time: f32) -> Vec3 {
    let sky_height = direction.y.max(0.).clamp(0., 1.);
    let shaped_height = sky_height.powf(0.58);
    let sun_direction = celestial_direction(cycle_time);
    let moon_direction = -sun_direction;
    let daylight = smoothstep(-0.14, 0.16, sun_direction.y);
    let night = 1. - smoothstep(-0.18, 0.05, sun_direction.y);
    let twilight = twilight_strength(sun_direction.y);
    let dusk = dusk_amount(cycle_time);

    let night_horizon = Vec3::new(0.025, 0.035, 0.095);
    let night_zenith = Vec3::new(0.0025, 0.006, 0.025);
    let day_horizon = Vec3::new(0.70, 0.86, 1.0);
    let day_zenith = Vec3::new(0.055, 0.25, 0.68);
    let horizon = mix(night_horizon, day_horizon, daylight);
    let zenith = mix(night_zenith, day_zenith, daylight);
    let mut color = mix(horizon, zenith, shaped_height);

    // The most saturated color stays close to the horizon and becomes more
    // intense toward the sun, yielding distinct lavender dawns and fiery dusk.
    let horizontal_sun = Vec3::new(sun_direction.x, 0., sun_direction.z).unit();
    let view_horizontal = Vec3::new(direction.x, 0., direction.z).unit();
    let toward_sun = ((view_horizontal.dot(&horizontal_sun) + 1.) * 0.5).powf(1.7);
    let horizon_band = (1. - sky_height).powf(3.2);
    let dawn_low = Vec3::new(1.0, 0.50, 0.30);
    let dusk_low = Vec3::new(1.0, 0.20, 0.075);
    let dawn_high = Vec3::new(0.48, 0.33, 0.72);
    let dusk_high = Vec3::new(0.50, 0.12, 0.38);
    let low_twilight = mix(dawn_low, dusk_low, dusk);
    let high_twilight = mix(dawn_high, dusk_high, dusk);
    let twilight_color = mix(
        low_twilight,
        high_twilight,
        smoothstep(0.0, 0.48, sky_height),
    );
    let twilight_amount = twilight * horizon_band * (0.38 + toward_sun * 0.62);
    color = mix(color, twilight_color, twilight_amount * 0.82);
    color += low_twilight * twilight * toward_sun.powf(3.) * horizon_band * 0.42;

    if direction.y > 0. {
        color += stars(direction) * night * smoothstep(0.015, 0.20, direction.y);
    }

    if sun_direction.y > -0.09 {
        let sun_dot = direction.dot(&sun_direction).clamp(0., 1.);
        let sun_disk = smoothstep(0.9985, 0.99955, sun_dot);
        let sun_glow = sun_dot.powf(if twilight > 0.2 { 18. } else { 30. });
        let sun_color = mix(
            Vec3::new(1.0, 0.35, 0.09),
            Vec3::new(1.0, 0.93, 0.72),
            daylight,
        );
        color += sun_color * sun_glow * (0.25 + twilight * 0.45);
        color += sun_color * sun_disk * 3.8;
    }

    if moon_direction.y > -0.07 {
        let moon_dot = direction.dot(&moon_direction).clamp(0., 1.);
        let moon_disk = smoothstep(0.99855, 0.99945, moon_dot);
        let moon_glow = moon_dot.powf(165.);
        let moon_visibility = (night + twilight * 0.42).clamp(0., 1.);
        color += Vec3::new(0.34, 0.48, 1.0) * moon_glow * 0.52 * moon_visibility;
        color += Vec3::new(0.88, 0.93, 1.0) * moon_disk * 2.7 * moon_visibility;
    }

    let clouds = cloud_density(direction) * mix_scalar(0.58, 1.0, daylight);
    if clouds > 0. {
        let sun_rim = direction.dot(&sun_direction).max(0.).powf(6.);
        let moon_rim = direction.dot(&moon_direction).max(0.).powf(8.) * night;
        let day_cloud = Vec3::new(0.76, 0.83, 0.91);
        let night_cloud = Vec3::new(0.055, 0.07, 0.14);
        let twilight_cloud = mix(
            Vec3::new(0.95, 0.46, 0.32),
            Vec3::new(0.82, 0.20, 0.27),
            dusk,
        );
        let mut cloud_color = mix(night_cloud, day_cloud, daylight);
        cloud_color = mix(
            cloud_color,
            twilight_cloud,
            twilight * (0.45 + sun_rim * 0.45),
        );
        cloud_color += Vec3::new(0.22, 0.28, 0.55) * moon_rim;
        color = color * (1. - clouds * 0.68) + cloud_color * clouds * 0.90;
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
        let directions = [
            Vec3::new(0., 1., 0.),
            Vec3::new(0.22, 0.96, 0.17).normalize(),
            Vec3::new(-0.31, 0.91, -0.27).normalize(),
            Vec3::new(0.42, 0.86, -0.29).normalize(),
        ];
        let day_brightness: f32 = directions
            .iter()
            .map(|direction| skybox(*direction, DAY_PHASE).norm())
            .sum();
        let night_brightness: f32 = directions
            .iter()
            .map(|direction| skybox(*direction, NIGHT_PHASE).norm())
            .sum();
        assert!(
            day_brightness > night_brightness * 2.5,
            "day={day_brightness}, night={night_brightness}"
        );
    }

    #[test]
    fn moon_is_bright_and_blue_white() {
        let moon_direction = -celestial_direction(NIGHT_PHASE);
        let moon = skybox(moon_direction, NIGHT_PHASE);
        assert!(moon.x > 0.8);
        assert!(moon.z >= moon.x);
    }

    #[test]
    fn sun_and_moon_follow_opposite_arcs() {
        let sunset_sun = celestial_direction(0.49);
        let sunset_moon = -sunset_sun;
        assert!(sunset_sun.y > 0.);
        assert!(sunset_moon.y < 0.);

        let later_moon = -celestial_direction(0.56);
        assert!(later_moon.y > 0.);
    }

    #[test]
    fn twilight_has_warmer_horizon_than_midday() {
        let west = celestial_direction(0.50);
        let sunset = skybox(west, 0.50);
        let midday = skybox(west, DAY_PHASE);
        assert!(sunset.x > sunset.z);
        assert!(sunset.x > midday.x);
    }
}
