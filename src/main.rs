//! Minecraft-inspired voxel ray tracer.
//! Uses `nalgebra` for vectors, `minifb` for the interactive window and `image`
//! for lossless PNG/BMP exports. The acceleration structure and renderer remain
//! implemented in this project.

use std::{cmp::Ordering, env, time::Instant};

use image::{Rgb, RgbImage};
use minifb::{Key, KeyRepeat, Scale, Window, WindowOptions};
use nalgebra::Vector3;

type Vec3 = Vector3<f32>;

const EPSILON: f32 = 0.002;
const MAX_BOUNCES: u32 = 3;
const LEAF: usize = usize::MAX;

trait Vec3Ext {
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
struct Ray {
    origin: Vec3,
    direction: Vec3,
}
impl Ray {
    fn at(self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}

#[derive(Clone, Copy)]
enum Texture {
    Grass,
    Dirt,
    Stone,
    Wood,
    Leaves,
    Water,
    Glass,
}

#[derive(Clone, Copy)]
struct Material {
    texture: Texture,
    albedo: Vec3,
    specular: f32,
    shininess: f32,
    reflectivity: f32,
    transparency: f32,
    ior: f32,
}

#[derive(Clone, Copy)]
struct Cube {
    min: Vec3,
    max: Vec3,
    material: usize,
}
#[derive(Clone, Copy)]
struct Hit {
    t: f32,
    point: Vec3,
    normal: Vec3,
    material: usize,
    uv: (f32, f32),
}
#[derive(Clone, Copy)]
struct Aabb {
    min: Vec3,
    max: Vec3,
}
#[derive(Clone, Copy)]
struct BvhNode {
    bounds: Aabb,
    left: usize,
    right: usize,
    start: usize,
    count: usize,
}

impl Aabb {
    fn union(a: Self, b: Self) -> Self {
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
    fn entry(self, ray: Ray, max_t: f32) -> Option<f32> {
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
    fn bounds(self) -> Aabb {
        Aabb {
            min: self.min,
            max: self.max,
        }
    }
    fn intersect(self, ray: Ray, t_max: f32) -> Option<Hit> {
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
        let p = ray.at(t);
        let edge = 0.003;
        let (normal, uv) = if (p.x - self.min.x).abs() < edge {
            (Vec3::new(-1., 0., 0.), (p.z, p.y))
        } else if (p.x - self.max.x).abs() < edge {
            (Vec3::new(1., 0., 0.), (p.z, p.y))
        } else if (p.y - self.min.y).abs() < edge {
            (Vec3::new(0., -1., 0.), (p.x, p.z))
        } else if (p.y - self.max.y).abs() < edge {
            (Vec3::new(0., 1., 0.), (p.x, p.z))
        } else if (p.z - self.min.z).abs() < edge {
            (Vec3::new(0., 0., -1.), (p.x, p.y))
        } else {
            (Vec3::new(0., 0., 1.), (p.x, p.y))
        };
        Some(Hit {
            t,
            point: p,
            normal,
            material: self.material,
            uv,
        })
    }
}

struct Scene {
    cubes: Vec<Cube>,
    materials: Vec<Material>,
    nodes: Vec<BvhNode>,
    order: Vec<usize>,
    root: usize,
}
impl Scene {
    fn add_cube(&mut self, x: f32, y: f32, z: f32, material: usize) {
        self.cubes.push(Cube {
            min: Vec3::new(x, y, z),
            max: Vec3::new(x + 1., y + 1., z + 1.),
            material,
        });
    }
    fn rebuild_bvh(&mut self) {
        self.nodes.clear();
        self.order.clear();
        let mut indices: Vec<usize> = (0..self.cubes.len()).collect();
        self.root = self.build_node(&mut indices);
    }
    fn build_node(&mut self, indices: &mut [usize]) -> usize {
        let mut bounds = self.cubes[indices[0]].bounds();
        for &index in &indices[1..] {
            bounds = Aabb::union(bounds, self.cubes[index].bounds());
        }
        if indices.len() <= 8 {
            let start = self.order.len();
            self.order.extend_from_slice(indices);
            let index = self.nodes.len();
            self.nodes.push(BvhNode {
                bounds,
                left: LEAF,
                right: LEAF,
                start,
                count: indices.len(),
            });
            return index;
        }
        let extent = bounds.max - bounds.min;
        let axis = if extent.x >= extent.y && extent.x >= extent.z {
            0
        } else if extent.y >= extent.z {
            1
        } else {
            2
        };
        indices.sort_by(|a, b| {
            let ca = (self.cubes[*a].min[axis] + self.cubes[*a].max[axis]) * 0.5;
            let cb = (self.cubes[*b].min[axis] + self.cubes[*b].max[axis]) * 0.5;
            ca.partial_cmp(&cb).unwrap_or(Ordering::Equal)
        });
        let middle = indices.len() / 2;
        let (first, second) = indices.split_at_mut(middle);
        let left = self.build_node(first);
        let right = self.build_node(second);
        let index = self.nodes.len();
        self.nodes.push(BvhNode {
            bounds,
            left,
            right,
            start: 0,
            count: 0,
        });
        index
    }
    fn hit(&self, ray: Ray, max_t: f32) -> Option<Hit> {
        let mut closest = max_t;
        let mut result = None;
        let mut stack = vec![self.root];
        while let Some(node_index) = stack.pop() {
            let node = self.nodes[node_index];
            if node.bounds.entry(ray, closest).is_none() {
                continue;
            }
            if node.left == LEAF {
                for &cube_index in &self.order[node.start..node.start + node.count] {
                    if let Some(hit) = self.cubes[cube_index].intersect(ray, closest) {
                        closest = hit.t;
                        result = Some(hit);
                    }
                }
            } else {
                let left = self.nodes[node.left].bounds.entry(ray, closest);
                let right = self.nodes[node.right].bounds.entry(ray, closest);
                match (left, right) {
                    (Some(l), Some(r)) => {
                        if l < r {
                            stack.push(node.right);
                            stack.push(node.left);
                        } else {
                            stack.push(node.left);
                            stack.push(node.right);
                        }
                    }
                    (Some(_), None) => stack.push(node.left),
                    (None, Some(_)) => stack.push(node.right),
                    _ => {}
                }
            }
        }
        result
    }
    /// Shadow rays only need any blocker; unlike `hit`, this exits immediately.
    fn occluded(&self, ray: Ray) -> bool {
        let mut stack = vec![self.root];
        while let Some(node_index) = stack.pop() {
            let node = self.nodes[node_index];
            if node.bounds.entry(ray, f32::INFINITY).is_none() {
                continue;
            }
            if node.left == LEAF {
                for &cube_index in &self.order[node.start..node.start + node.count] {
                    if self.cubes[cube_index]
                        .intersect(ray, f32::INFINITY)
                        .is_some()
                    {
                        return true;
                    }
                }
            } else {
                stack.push(node.left);
                stack.push(node.right);
            }
        }
        false
    }
}

fn fract(x: f32) -> f32 {
    x - x.floor()
}
fn hash(x: f32, y: f32) -> f32 {
    fract((x * 127.1 + y * 311.7).sin() * 43758.545)
}
fn checker(u: f32, v: f32, scale: f32) -> bool {
    ((u * scale).floor() as i32 + (v * scale).floor() as i32) & 1 == 0
}
fn texture(kind: Texture, u: f32, v: f32) -> Vec3 {
    let u = fract(u);
    let v = fract(v);
    let grain = hash((u * 32.).floor(), (v * 32.).floor());
    match kind {
        Texture::Grass => {
            let blades = (u * 18. + v * 4.).sin() * 0.06;
            Vec3::new(0.20 + blades, 0.48 + grain * 0.16, 0.12 + blades * 0.3)
        }
        Texture::Dirt => Vec3::new(
            0.30 + grain * 0.16,
            0.16 + grain * 0.09,
            0.065 + grain * 0.035,
        ),
        Texture::Stone => {
            let c = 0.30 + grain * 0.22 + if checker(u, v, 4.) { 0.04 } else { 0.0 };
            Vec3::new(c * 0.88, c * 0.92, c)
        }
        Texture::Wood => {
            let rings = ((u * 17. + (v * 8.).sin() * 1.5).sin() * 0.5 + 0.5) * 0.20;
            Vec3::new(0.27 + rings, 0.105 + rings * 0.48, 0.028 + rings * 0.20)
        }
        Texture::Leaves => Vec3::new(
            0.055 + grain * 0.07,
            0.23 + grain * 0.20,
            0.045 + grain * 0.06,
        ),
        Texture::Water => Vec3::new(
            0.015,
            0.20 + (u * 20. + v * 13.).sin() * 0.025,
            0.33 + (u * 14.).sin() * 0.025,
        ),
        Texture::Glass => Vec3::new(0.66, 0.89, 0.91),
    }
}
fn skybox(d: Vec3) -> Vec3 {
    let h = (d.y * 0.5 + 0.5).clamp(0., 1.);
    let mut color = Vec3::new(0.76, 0.87, 0.98) * (1. - h) + Vec3::new(0.12, 0.36, 0.72) * h;
    if d.y > 0.05 {
        let cloud = hash((d.x * 18.).floor(), (d.z * 18.).floor());
        if cloud > 0.78 {
            color += Vec3::new(0.18, 0.18, 0.16) * ((cloud - 0.78) / 0.22) * h;
        }
    }
    let sun = d
        .dot(&Vec3::new(-0.45, 0.72, -0.52).unit())
        .max(0.)
        .powf(300.);
    color + Vec3::new(1., 0.78, 0.42) * sun * 3.
}
fn reflect(d: Vec3, n: Vec3) -> Vec3 {
    d - n * (2. * d.dot(&n))
}
fn refract(d: Vec3, n: Vec3, ior: f32) -> Option<Vec3> {
    let eta = if d.dot(&n) < 0. { 1. / ior } else { ior };
    let nn = if d.dot(&n) < 0. { n } else { -n };
    let cos_i = (-d).dot(&nn).clamp(0., 1.);
    let k = 1. - eta * eta * (1. - cos_i * cos_i);
    if k < 0. {
        None
    } else {
        Some((d * eta + nn * (eta * cos_i - k.sqrt())).unit())
    }
}
fn trace(scene: &Scene, ray: Ray, depth: u32, sun_dir: Vec3) -> Vec3 {
    if depth >= MAX_BOUNCES {
        return skybox(ray.direction);
    }
    let Some(hit) = scene.hit(ray, f32::INFINITY) else {
        return skybox(ray.direction);
    };
    let material = scene.materials[hit.material];
    let albedo = texture(material.texture, hit.uv.0, hit.uv.1).hadamard(material.albedo);
    let shadow = scene.occluded(Ray {
        origin: hit.point + hit.normal * EPSILON,
        direction: sun_dir,
    });
    let diffuse = hit.normal.dot(&sun_dir).max(0.) * if shadow { 0.14 } else { 1.0 };
    let halfway = (sun_dir - ray.direction).unit();
    let specular = hit.normal.dot(&halfway).max(0.).powf(material.shininess)
        * material.specular
        * if shadow { 0. } else { 1. };
    let mut local = albedo * (0.16 + diffuse * 0.84) + Vec3::new(1., 0.91, 0.75) * specular;
    let mut reflected = Vec3::zeros();
    if material.reflectivity > 0. || material.transparency > 0. {
        reflected = trace(
            scene,
            Ray {
                origin: hit.point + hit.normal * EPSILON,
                direction: reflect(ray.direction, hit.normal).unit(),
            },
            depth + 1,
            sun_dir,
        );
    }
    if material.transparency > 0. {
        let transmitted = refract(ray.direction, hit.normal, material.ior)
            .map(|direction| {
                trace(
                    scene,
                    Ray {
                        origin: hit.point + direction * EPSILON,
                        direction,
                    },
                    depth + 1,
                    sun_dir,
                )
            })
            .unwrap_or(reflected);
        local = local * (1. - material.transparency)
            + transmitted.hadamard(albedo) * material.transparency;
    }
    local * (1. - material.reflectivity) + reflected * material.reflectivity
}

#[derive(Clone, Copy)]
struct Camera {
    target: Vec3,
    yaw: f32,
    pitch: f32,
    distance: f32,
    fov: f32,
}
#[derive(Clone, Copy)]
struct CameraFrame {
    origin: Vec3,
    forward: Vec3,
    right: Vec3,
    up: Vec3,
    view: f32,
    aspect: f32,
}
impl Camera {
    fn frame(&self, width: u32, height: u32) -> CameraFrame {
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
}
impl CameraFrame {
    fn ray(self, x: u32, y: u32, width: u32, height: u32) -> Ray {
        let sx = (((x as f32 + 0.5) / width as f32) * 2. - 1.) * self.aspect * self.view;
        let sy = (1. - ((y as f32 + 0.5) / height as f32) * 2.) * self.view;
        Ray {
            origin: self.origin,
            direction: (self.forward + self.right * sx + self.up * sy).unit(),
        }
    }
}

fn build_scene() -> Scene {
    let materials = vec![
        Material {
            texture: Texture::Grass,
            albedo: Vec3::new(1., 1., 1.),
            specular: 0.05,
            shininess: 12.,
            reflectivity: 0.,
            transparency: 0.,
            ior: 1.,
        },
        Material {
            texture: Texture::Dirt,
            albedo: Vec3::new(1., 1., 1.),
            specular: 0.02,
            shininess: 8.,
            reflectivity: 0.,
            transparency: 0.,
            ior: 1.,
        },
        Material {
            texture: Texture::Stone,
            albedo: Vec3::new(1., 1., 1.),
            specular: 0.16,
            shininess: 28.,
            reflectivity: 0.06,
            transparency: 0.,
            ior: 1.,
        },
        Material {
            texture: Texture::Wood,
            albedo: Vec3::new(1., 1., 1.),
            specular: 0.08,
            shininess: 16.,
            reflectivity: 0.,
            transparency: 0.,
            ior: 1.,
        },
        Material {
            texture: Texture::Leaves,
            albedo: Vec3::new(1., 1., 1.),
            specular: 0.03,
            shininess: 8.,
            reflectivity: 0.,
            transparency: 0.,
            ior: 1.,
        },
        Material {
            texture: Texture::Water,
            albedo: Vec3::new(1., 1., 1.),
            specular: 0.75,
            shininess: 95.,
            reflectivity: 0.24,
            transparency: 0.48,
            ior: 1.333,
        },
        Material {
            texture: Texture::Glass,
            albedo: Vec3::new(0.86, 0.96, 1.),
            specular: 0.9,
            shininess: 120.,
            reflectivity: 0.12,
            transparency: 0.72,
            ior: 1.52,
        },
    ];
    let mut scene = Scene {
        cubes: Vec::new(),
        materials,
        nodes: Vec::new(),
        order: Vec::new(),
        root: 0,
    };
    for x in -10..=10 {
        for z in -10..=10 {
            scene.add_cube(x as f32, -2., z as f32, 2);
            let lake = (-4..=4).contains(&x) && (2..=6).contains(&z);
            let path = (x == -1 || x == 0) && z < 2 && z > -10;
            scene.add_cube(
                x as f32,
                -1.,
                z as f32,
                if lake {
                    5
                } else if path {
                    2
                } else {
                    0
                },
            );
        }
    }
    for y in 0..4 {
        for x in -5..=-1 {
            for z in -4..=0 {
                if x == -5 || x == -1 || z == -4 || z == 0 {
                    if z == 0 && y == 2 && (x == -4 || x == -2) {
                        scene.add_cube(x as f32, y as f32, z as f32, 6);
                    } else {
                        scene.add_cube(
                            x as f32,
                            y as f32,
                            z as f32,
                            if (x + z + y) % 3 == 0 { 3 } else { 1 },
                        );
                    }
                }
            }
        }
    }
    for x in -6i32..=0 {
        for z in -5..=1 {
            if (x + z).abs() % 2 == 0 {
                scene.add_cube(x as f32, 4., z as f32, 4);
            }
        }
    }
    for y in 0..6 {
        scene.add_cube(-5., y as f32, -3., 2);
    }
    for y in 0..3 {
        for x in 2..=5 {
            for z in 0..=2 {
                if x == 2 || x == 5 || z == 0 || z == 2 {
                    scene.add_cube(
                        x as f32,
                        y as f32,
                        z as f32,
                        if (x + y + z) % 3 == 0 { 3 } else { 6 },
                    );
                }
            }
        }
    }
    for x in 2..=5 {
        for z in 0..=2 {
            scene.add_cube(x as f32, 3., z as f32, 6);
        }
    }
    for &(tx, tz) in &[(-8, -6), (7, -5), (7, 5)] {
        for y in 0..4 {
            scene.add_cube(tx as f32, y as f32, tz as f32, 3);
        }
        for x in tx - 1..=tx + 1 {
            for y in 3..=5 {
                for z in tz - 1..=tz + 1 {
                    if !(x == tx && y == 3 && z == tz) {
                        scene.add_cube(x as f32, y as f32, z as f32, 4);
                    }
                }
            }
        }
        scene.add_cube(tx as f32, 6., tz as f32, 4);
    }
    for &(x, z) in &[(-5, 2), (-5, 5), (5, 2), (5, 6), (-2, 7)] {
        scene.add_cube(x as f32, 0., z as f32, 2);
    }
    scene.rebuild_bvh();
    scene
}

fn to_pixel(color: Vec3) -> u32 {
    let c = color.clamp_rgb();
    let r = (c.x.powf(1. / 2.2) * 255.) as u32;
    let g = (c.y.powf(1. / 2.2) * 255.) as u32;
    let b = (c.z.powf(1. / 2.2) * 255.) as u32;
    (r << 16) | (g << 8) | b
}
fn render(scene: &Scene, camera: Camera, width: u32, height: u32) -> Vec<u32> {
    let started = Instant::now();
    let frame = camera.frame(width, height);
    let sun = Vec3::new(-0.45, 0.72, -0.52).unit();
    let mut pixels = vec![0; (width * height) as usize];
    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .min(height as usize)
        .max(1);
    let rows_per_thread = (height as usize).div_ceil(threads);
    std::thread::scope(|scope| {
        for (chunk_index, rows) in pixels
            .chunks_mut(rows_per_thread * width as usize)
            .enumerate()
        {
            scope.spawn(move || {
                for (local_y, row) in rows.chunks_mut(width as usize).enumerate() {
                    let y = (chunk_index * rows_per_thread + local_y) as u32;
                    for (x, pixel) in row.iter_mut().enumerate() {
                        *pixel =
                            to_pixel(trace(scene, frame.ray(x as u32, y, width, height), 0, sun));
                    }
                }
            });
        }
    });
    println!(
        "Render {}×{} | {} cubos | {} hilos | {:.2?}",
        width,
        height,
        scene.cubes.len(),
        threads,
        started.elapsed()
    );
    pixels
}
fn save_image(path: &str, pixels: &[u32], width: u32, height: u32) -> image::ImageResult<()> {
    let mut image = RgbImage::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let pixel = pixels[(y * width + x) as usize];
            image.put_pixel(
                x,
                y,
                Rgb([
                    ((pixel >> 16) & 255) as u8,
                    ((pixel >> 8) & 255) as u8,
                    (pixel & 255) as u8,
                ]),
            );
        }
    }
    image.save(path)
}
fn parse(args: &[String], name: &str, default: f32) -> f32 {
    args.iter()
        .position(|arg| arg == name)
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}
fn usage() {
    println!("cargo run --release -- [--width 480] [--height 320] [--yaw 35] [--pitch 22] [--distance 26] [--output diorama.png]");
}

fn interactive(scene: Scene, mut camera: Camera, width: u32, height: u32) {
    let mut window = Window::new(
        "Diorama voxel | ← → rota | ↑ ↓ inclina | W/S zoom | P captura PNG | Esc salir",
        width as usize,
        height as usize,
        WindowOptions {
            resize: false,
            scale: Scale::X2,
            ..WindowOptions::default()
        },
    )
    .expect("Could not create minifb window");
    let mut pixels = render(&scene, camera, width, height);
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mut changed = false;
        if window.is_key_pressed(Key::Left, KeyRepeat::Yes)
            || window.is_key_pressed(Key::A, KeyRepeat::Yes)
        {
            camera.yaw -= 8.;
            changed = true;
        }
        if window.is_key_pressed(Key::Right, KeyRepeat::Yes)
            || window.is_key_pressed(Key::D, KeyRepeat::Yes)
        {
            camera.yaw += 8.;
            changed = true;
        }
        if window.is_key_pressed(Key::Up, KeyRepeat::Yes) {
            camera.pitch = (camera.pitch + 4.).clamp(-5., 75.);
            changed = true;
        }
        if window.is_key_pressed(Key::Down, KeyRepeat::Yes) {
            camera.pitch = (camera.pitch - 4.).clamp(-5., 75.);
            changed = true;
        }
        if window.is_key_pressed(Key::W, KeyRepeat::Yes) {
            camera.distance = (camera.distance - 2.).max(8.);
            changed = true;
        }
        if window.is_key_pressed(Key::S, KeyRepeat::Yes) {
            camera.distance = (camera.distance + 2.).min(60.);
            changed = true;
        }
        if window.is_key_pressed(Key::P, KeyRepeat::No) {
            match save_image("diorama.png", &pixels, width, height) {
                Ok(_) => println!("Captura guardada como diorama.png"),
                Err(error) => eprintln!("No se pudo guardar: {error}"),
            }
        }
        if changed {
            pixels = render(&scene, camera, width, height);
            window.set_title(&format!(
                "Diorama | yaw {:.0}° | distancia {:.0} | ← → rota | W/S zoom | P captura",
                camera.yaw, camera.distance
            ));
        }
        window
            .update_with_buffer(&pixels, width as usize, height as usize)
            .expect("Could not update window");
    }
}
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        usage();
        return;
    }
    let width = parse(&args, "--width", 480.).max(32.) as u32;
    let height = parse(&args, "--height", 320.).max(32.) as u32;
    let camera = Camera {
        target: Vec3::new(0., 1., 0.),
        yaw: parse(&args, "--yaw", 35.),
        pitch: parse(&args, "--pitch", 22.),
        distance: parse(&args, "--distance", 26.),
        fov: 52.,
    };
    let scene = build_scene();
    if let Some(path) = args
        .iter()
        .position(|arg| arg == "--output")
        .and_then(|i| args.get(i + 1))
    {
        let pixels = render(&scene, camera, width, height);
        save_image(path, &pixels, width, height).expect("Could not save image");
        println!("Imagen guardada: {path}");
    } else {
        interactive(scene, camera, width, height);
    }
}
