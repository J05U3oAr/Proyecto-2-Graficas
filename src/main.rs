//! A small, dependency-free Minecraft-inspired cube diorama renderer.
//! Output is a binary PPM (`P6`) file, which can be opened by most image viewers
//! or converted with a tool of the student's choice.

use std::{env, ffi::c_void, fs::File, io::Write, time::Instant};

const EPSILON: f32 = 0.002;
const MAX_BOUNCES: u32 = 3;

#[derive(Clone, Copy, Debug, Default)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    fn dot(self, b: Self) -> f32 {
        self.x * b.x + self.y * b.y + self.z * b.z
    }
    fn cross(self, b: Self) -> Self {
        Self::new(
            self.y * b.z - self.z * b.y,
            self.z * b.x - self.x * b.z,
            self.x * b.y - self.y * b.x,
        )
    }
    fn len(self) -> f32 {
        self.dot(self).sqrt()
    }
    fn unit(self) -> Self {
        let l = self.len();
        if l > 0.0 {
            self / l
        } else {
            self
        }
    }
    fn hadamard(self, b: Self) -> Self {
        Self::new(self.x * b.x, self.y * b.y, self.z * b.z)
    }
    fn clamp(self, low: f32, high: f32) -> Self {
        Self::new(
            self.x.clamp(low, high),
            self.y.clamp(low, high),
            self.z.clamp(low, high),
        )
    }
}
use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub};
impl Add for Vec3 {
    type Output = Self;
    fn add(self, b: Self) -> Self {
        Self::new(self.x + b.x, self.y + b.y, self.z + b.z)
    }
}
impl AddAssign for Vec3 {
    fn add_assign(&mut self, b: Self) {
        *self = *self + b;
    }
}
impl Sub for Vec3 {
    type Output = Self;
    fn sub(self, b: Self) -> Self {
        Self::new(self.x - b.x, self.y - b.y, self.z - b.z)
    }
}
impl Mul<f32> for Vec3 {
    type Output = Self;
    fn mul(self, n: f32) -> Self {
        Self::new(self.x * n, self.y * n, self.z * n)
    }
}
impl Mul<Vec3> for f32 {
    type Output = Vec3;
    fn mul(self, v: Vec3) -> Vec3 {
        v * self
    }
}
impl Div<f32> for Vec3 {
    type Output = Self;
    fn div(self, n: f32) -> Self {
        Self::new(self.x / n, self.y / n, self.z / n)
    }
}
impl Neg for Vec3 {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
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

/// Each material has independent texture and BRDF/transmission parameters.
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

impl Cube {
    fn intersect(&self, ray: Ray, t_max: f32) -> Option<Hit> {
        let inv = Vec3::new(
            1.0 / ray.direction.x,
            1.0 / ray.direction.y,
            1.0 / ray.direction.z,
        );
        let tx1 = (self.min.x - ray.origin.x) * inv.x;
        let tx2 = (self.max.x - ray.origin.x) * inv.x;
        let ty1 = (self.min.y - ray.origin.y) * inv.y;
        let ty2 = (self.max.y - ray.origin.y) * inv.y;
        let tz1 = (self.min.z - ray.origin.z) * inv.z;
        let tz2 = (self.max.z - ray.origin.z) * inv.z;
        let near = tx1.min(tx2).max(ty1.min(ty2)).max(tz1.min(tz2));
        let far = tx1.max(tx2).min(ty1.max(ty2)).min(tz1.max(tz2));
        if far < near || far < EPSILON || near > t_max {
            return None;
        }
        let t = if near > EPSILON { near } else { far };
        let p = ray.at(t);
        let d = 0.003;
        let (normal, uv) = if (p.x - self.min.x).abs() < d {
            (Vec3::new(-1., 0., 0.), (p.z, p.y))
        } else if (p.x - self.max.x).abs() < d {
            (Vec3::new(1., 0., 0.), (p.z, p.y))
        } else if (p.y - self.min.y).abs() < d {
            (Vec3::new(0., -1., 0.), (p.x, p.z))
        } else if (p.y - self.max.y).abs() < d {
            (Vec3::new(0., 1., 0.), (p.x, p.z))
        } else if (p.z - self.min.z).abs() < d {
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
}
impl Scene {
    fn hit(&self, ray: Ray, max_t: f32) -> Option<Hit> {
        let mut closest = max_t;
        let mut result = None;
        for cube in &self.cubes {
            if let Some(hit) = cube.intersect(ray, closest) {
                closest = hit.t;
                result = Some(hit);
            }
        }
        result
    }
    fn add_cube(&mut self, x: f32, y: f32, z: f32, material: usize) {
        self.cubes.push(Cube {
            min: Vec3::new(x, y, z),
            max: Vec3::new(x + 1., y + 1., z + 1.),
            material,
        });
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

/// Procedural textures are intentionally stored in code so the project needs no image crate.
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
    // Gradient, sun halo and value-noise clouds make this an actual direction-based skybox.
    let h = (d.y * 0.5 + 0.5).clamp(0., 1.);
    let horizon = Vec3::new(0.76, 0.87, 0.98);
    let zenith = Vec3::new(0.12, 0.36, 0.72);
    let mut color = horizon * (1.0 - h) + zenith * h;
    if d.y > 0.05 {
        let cloud = hash((d.x * 18.).floor(), (d.z * 18.).floor());
        if cloud > 0.78 {
            color += Vec3::new(0.18, 0.18, 0.16) * ((cloud - 0.78) / 0.22) * h;
        }
    }
    let sun = d
        .dot(Vec3::new(-0.45, 0.72, -0.52).unit())
        .max(0.)
        .powf(300.);
    color + Vec3::new(1.0, 0.78, 0.42) * sun * 3.0
}

fn reflect(d: Vec3, n: Vec3) -> Vec3 {
    d - n * (2.0 * d.dot(n))
}
fn refract(d: Vec3, n: Vec3, ior: f32) -> Option<Vec3> {
    let eta = if d.dot(n) < 0.0 { 1.0 / ior } else { ior };
    let nn = if d.dot(n) < 0.0 { n } else { -n };
    let cos_i = (-d).dot(nn).clamp(0., 1.);
    let k = 1.0 - eta * eta * (1.0 - cos_i * cos_i);
    if k < 0.0 {
        None
    } else {
        Some((d * eta + nn * (eta * cos_i - k.sqrt())).unit())
    }
}

fn trace(scene: &Scene, ray: Ray, depth: u32) -> Vec3 {
    if depth >= MAX_BOUNCES {
        return skybox(ray.direction);
    }
    let Some(hit) = scene.hit(ray, f32::INFINITY) else {
        return skybox(ray.direction);
    };
    let mat = scene.materials[hit.material];
    let albedo = texture(mat.texture, hit.uv.0, hit.uv.1).hadamard(mat.albedo);
    let sun_dir = Vec3::new(-0.45, 0.72, -0.52).unit();
    let shadow_ray = Ray {
        origin: hit.point + hit.normal * EPSILON,
        direction: sun_dir,
    };
    let in_shadow = scene.hit(shadow_ray, f32::INFINITY).is_some();
    let diffuse = hit.normal.dot(sun_dir).max(0.0) * if in_shadow { 0.14 } else { 1.0 };
    let view = -ray.direction;
    let halfway = (sun_dir + view).unit();
    let spec = hit.normal.dot(halfway).max(0.0).powf(mat.shininess)
        * mat.specular
        * if in_shadow { 0.0 } else { 1.0 };
    let mut local = albedo * (0.16 + diffuse * 0.84) + Vec3::new(1., 0.91, 0.75) * spec;

    let mut reflected = Vec3::default();
    if mat.reflectivity > 0.0 || mat.transparency > 0.0 {
        let reflected_ray = Ray {
            origin: hit.point + hit.normal * EPSILON,
            direction: reflect(ray.direction, hit.normal).unit(),
        };
        reflected = trace(scene, reflected_ray, depth + 1);
    }
    if mat.transparency > 0.0 {
        let transmitted = refract(ray.direction, hit.normal, mat.ior)
            .map(|d| {
                trace(
                    scene,
                    Ray {
                        origin: hit.point + d * EPSILON,
                        direction: d,
                    },
                    depth + 1,
                )
            })
            .unwrap_or(reflected);
        // A small tint keeps water/glass readable while showing the refracted scene behind them.
        local = local * (1.0 - mat.transparency) + transmitted.hadamard(albedo) * mat.transparency;
    }
    local * (1.0 - mat.reflectivity) + reflected * mat.reflectivity
}

struct Camera {
    target: Vec3,
    yaw: f32,
    pitch: f32,
    distance: f32,
    fov: f32,
}
impl Camera {
    fn ray(&self, px: f32, py: f32, width: u32, height: u32) -> Ray {
        let yaw = self.yaw.to_radians();
        let pitch = self.pitch.to_radians();
        let position = self.target
            + Vec3::new(
                pitch.cos() * yaw.cos(),
                pitch.sin(),
                pitch.cos() * yaw.sin(),
            ) * self.distance;
        let forward = (self.target - position).unit();
        let right = forward.cross(Vec3::new(0., 1., 0.)).unit();
        let up = right.cross(forward).unit();
        let aspect = width as f32 / height as f32;
        let view = (self.fov.to_radians() * 0.5).tan();
        let sx = ((px / width as f32) * 2.0 - 1.0) * aspect * view;
        let sy = (1.0 - (py / height as f32) * 2.0) * view;
        Ray {
            origin: position,
            direction: (forward + right * sx + up * sy).unit(),
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
    };
    // A raised 21x21 stone/grass island with a lake and a winding stone path.
    for x in -10..=10 {
        for z in -10..=10 {
            scene.add_cube(x as f32, -2., z as f32, 2);
            let lake = (-4..=4).contains(&x) && (2..=6).contains(&z);
            let path = (x == -1 || x == 0) && z < 2 && z > -10;
            if lake {
                scene.add_cube(x as f32, -1., z as f32, 5);
            } else {
                scene.add_cube(x as f32, -1., z as f32, if path { 2 } else { 0 });
            }
        }
    }
    // Cabin: timber frame, stone chimney, glass windows, and a leaf roof.
    for y in 0..4 {
        for x in -5..=-1 {
            for z in -4..=0 {
                let edge = x == -5 || x == -1 || z == -4 || z == 0;
                if edge {
                    let m = if (x + z + y) % 3 == 0 { 3 } else { 1 };
                    // two front windows facing the path
                    if z == 0 && y == 2 && (x == -4 || x == -2) {
                        scene.add_cube(x as f32, y as f32, z as f32, 6);
                    } else {
                        scene.add_cube(x as f32, y as f32, z as f32, m);
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
    // Small greenhouse beside the lake (glass makes refraction obvious).
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
    // Three voxel trees with overlapping leaf canopies.
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
    // Boulders around the water.
    for &(x, z) in &[(-5, 2), (-5, 5), (5, 2), (5, 6), (-2, 7)] {
        scene.add_cube(x as f32, 0., z as f32, 2);
    }
    scene
}

fn parse_arg(args: &[String], flag: &str, default: f32) -> f32 {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}
fn usage() {
    println!("Usage: cargo run --release -- [--width 480] [--height 320] [--yaw 35] [--pitch 22] [--distance 26]");
    println!("Without --output, opens an interactive Windows viewer.");
    println!("Use --output diorama.bmp to render an image and exit.");
}

fn save_image(path: &str, width: u32, height: u32, pixels: &[u8]) {
    let mut file = File::create(path).expect("could not create output image");
    if path.to_lowercase().ends_with(".bmp") {
        // 24-bit BMP is trivial to write and opens directly in Windows Photos/Paint.
        let row_bytes = ((width * 3 + 3) / 4) * 4;
        let image_bytes = row_bytes * height;
        let file_bytes = 54 + image_bytes;
        let mut header = [0u8; 54];
        header[0..2].copy_from_slice(b"BM");
        header[2..6].copy_from_slice(&file_bytes.to_le_bytes());
        header[10..14].copy_from_slice(&54u32.to_le_bytes());
        header[14..18].copy_from_slice(&40u32.to_le_bytes());
        header[18..22].copy_from_slice(&(width as i32).to_le_bytes());
        header[22..26].copy_from_slice(&(height as i32).to_le_bytes());
        header[26..28].copy_from_slice(&1u16.to_le_bytes());
        header[28..30].copy_from_slice(&24u16.to_le_bytes());
        header[34..38].copy_from_slice(&image_bytes.to_le_bytes());
        file.write_all(&header).unwrap();
        let padding = [0u8; 3];
        for y in (0..height).rev() {
            for x in 0..width {
                let i = ((y * width + x) * 3) as usize;
                // BMP stores BGR, while the renderer stores RGB.
                file.write_all(&[pixels[i + 2], pixels[i + 1], pixels[i]])
                    .unwrap();
            }
            file.write_all(&padding[..(row_bytes - width * 3) as usize])
                .unwrap();
        }
    } else {
        write!(file, "P6\n{} {}\n255\n", width, height).unwrap();
        file.write_all(pixels).unwrap();
    }
}

fn render_pixels(scene: &Scene, camera: &Camera, width: u32, height: u32) -> Vec<u8> {
    let camera = Camera {
        target: camera.target,
        yaw: camera.yaw,
        pitch: camera.pitch,
        distance: camera.distance,
        fov: camera.fov,
    };
    println!(
        "Rendering {} cubes at {}x{} ...",
        scene.cubes.len(),
        width,
        height
    );
    let started = Instant::now();
    let mut pixels = Vec::with_capacity((width * height * 3) as usize);
    for y in 0..height {
        for x in 0..width {
            // Centre sampling; increase this to a small loop later for anti-aliasing.
            let c = trace(
                &scene,
                camera.ray(x as f32 + 0.5, y as f32 + 0.5, width, height),
                0,
            )
            .clamp(0., 1.);
            // Gamma correction before 8-bit PPM conversion.
            for channel in [c.x, c.y, c.z] {
                pixels.push((channel.powf(1.0 / 2.2) * 255.0) as u8);
            }
        }
        if y % 32 == 0 {
            println!("  {}%", y * 100 / height);
        }
    }
    println!("Rendered in {:.2?}", started.elapsed());
    pixels
}

#[cfg(target_os = "windows")]
mod native_window {
    use super::*;

    type Hwnd = *mut c_void;
    type Hdc = *mut c_void;
    type Hinstance = *mut c_void;
    type Wparam = usize;
    type Lparam = isize;
    type Lresult = isize;
    type Uint = u32;
    type Dword = u32;
    type Bool = i32;
    type Atom = u16;

    const WM_DESTROY: Uint = 0x0002;
    const WM_PAINT: Uint = 0x000F;
    const WM_KEYDOWN: Uint = 0x0100;
    const WM_MOUSEWHEEL: Uint = 0x020A;
    const VK_ESCAPE: Wparam = 0x1B;
    const VK_LEFT: Wparam = 0x25;
    const VK_UP: Wparam = 0x26;
    const VK_RIGHT: Wparam = 0x27;
    const VK_DOWN: Wparam = 0x28;
    const WS_OVERLAPPEDWINDOW: Dword = 0x00CF0000;
    const CW_USEDEFAULT: i32 = 0x80000000u32 as i32;
    const SW_SHOW: i32 = 5;
    const GWLP_USERDATA: i32 = -21;
    const DIB_RGB_COLORS: Uint = 0;
    const SRCCOPY: Dword = 0x00CC0020;

    #[repr(C)]
    struct Point {
        x: i32,
        y: i32,
    }
    #[repr(C)]
    struct Rect {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }
    #[repr(C)]
    struct Msg {
        hwnd: Hwnd,
        message: Uint,
        w_param: Wparam,
        l_param: Lparam,
        time: Dword,
        point: Point,
        l_private: Dword,
    }
    #[repr(C)]
    struct PaintStruct {
        hdc: Hdc,
        erase: Bool,
        paint: Rect,
        restore: Bool,
        update: Bool,
        reserved: [u8; 32],
    }
    #[repr(C)]
    struct BitmapInfoHeader {
        size: Dword,
        width: i32,
        height: i32,
        planes: u16,
        bit_count: u16,
        compression: Dword,
        size_image: Dword,
        x_ppm: i32,
        y_ppm: i32,
        colors_used: Dword,
        colors_important: Dword,
    }
    #[repr(C)]
    struct BitmapInfo {
        header: BitmapInfoHeader,
        colors: [u8; 4],
    }
    type WndProc = Option<unsafe extern "system" fn(Hwnd, Uint, Wparam, Lparam) -> Lresult>;
    #[repr(C)]
    struct WndClassEx {
        size: Uint,
        style: Uint,
        proc: WndProc,
        class_extra: i32,
        window_extra: i32,
        instance: Hinstance,
        icon: *mut c_void,
        cursor: *mut c_void,
        background: *mut c_void,
        menu_name: *const u16,
        class_name: *const u16,
        icon_small: *mut c_void,
    }

    #[link(name = "user32")]
    unsafe extern "system" {
        fn GetModuleHandleW(name: *const u16) -> Hinstance;
        fn RegisterClassExW(class: *const WndClassEx) -> Atom;
        fn CreateWindowExW(
            ex_style: Dword,
            class: *const u16,
            title: *const u16,
            style: Dword,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
            parent: Hwnd,
            menu: *mut c_void,
            instance: Hinstance,
            param: *mut c_void,
        ) -> Hwnd;
        fn DefWindowProcW(hwnd: Hwnd, message: Uint, w_param: Wparam, l_param: Lparam) -> Lresult;
        fn DestroyWindow(hwnd: Hwnd) -> Bool;
        fn ShowWindow(hwnd: Hwnd, command: i32) -> Bool;
        fn UpdateWindow(hwnd: Hwnd) -> Bool;
        fn GetMessageW(msg: *mut Msg, hwnd: Hwnd, min: Uint, max: Uint) -> i32;
        fn TranslateMessage(msg: *const Msg) -> Bool;
        fn DispatchMessageW(msg: *const Msg) -> Lresult;
        fn PostQuitMessage(code: i32);
        fn BeginPaint(hwnd: Hwnd, paint: *mut PaintStruct) -> Hdc;
        fn EndPaint(hwnd: Hwnd, paint: *const PaintStruct) -> Bool;
        fn GetClientRect(hwnd: Hwnd, rect: *mut Rect) -> Bool;
        fn InvalidateRect(hwnd: Hwnd, rect: *const Rect, erase: Bool) -> Bool;
        fn GetWindowLongPtrW(hwnd: Hwnd, index: i32) -> isize;
        fn SetWindowLongPtrW(hwnd: Hwnd, index: i32, value: isize) -> isize;
        fn SetWindowTextW(hwnd: Hwnd, text: *const u16) -> Bool;
    }
    #[link(name = "gdi32")]
    unsafe extern "system" {
        fn StretchDIBits(
            hdc: Hdc,
            x: i32,
            y: i32,
            dest_width: i32,
            dest_height: i32,
            src_x: i32,
            src_y: i32,
            src_width: i32,
            src_height: i32,
            bits: *const c_void,
            info: *const BitmapInfo,
            usage: Uint,
            rop: Dword,
        ) -> i32;
    }

    struct RenderState {
        scene: Scene,
        camera: Camera,
        width: u32,
        height: u32,
        bgr_pixels: Vec<u8>,
    }
    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().chain(std::iter::once(0)).collect()
    }
    fn info(state: &RenderState) -> BitmapInfo {
        BitmapInfo {
            header: BitmapInfoHeader {
                size: std::mem::size_of::<BitmapInfoHeader>() as u32,
                width: state.width as i32,
                height: -(state.height as i32),
                planes: 1,
                bit_count: 24,
                compression: 0,
                size_image: 0,
                x_ppm: 0,
                y_ppm: 0,
                colors_used: 0,
                colors_important: 0,
            },
            colors: [0; 4],
        }
    }
    fn render_state(state: &mut RenderState) {
        let rgb = render_pixels(&state.scene, &state.camera, state.width, state.height);
        state.bgr_pixels.clear();
        state.bgr_pixels.reserve(rgb.len());
        for p in rgb.chunks_exact(3) {
            state.bgr_pixels.extend_from_slice(&[p[2], p[1], p[0]]);
        }
    }
    unsafe fn state_from(hwnd: Hwnd) -> Option<&'static mut RenderState> {
        let pointer = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut RenderState;
        unsafe { pointer.as_mut() }
    }
    unsafe fn update_title(hwnd: Hwnd, state: &RenderState) {
        let title = wide(&format!("Diorama | yaw {:.0}° | pitch {:.0}° | distancia {:.0} | ← → rota, ↑ ↓ inclina, W/S zoom, Esc salir", state.camera.yaw, state.camera.pitch, state.camera.distance));
        unsafe {
            SetWindowTextW(hwnd, title.as_ptr());
        }
    }
    unsafe extern "system" fn wnd_proc(
        hwnd: Hwnd,
        message: Uint,
        w_param: Wparam,
        _l_param: Lparam,
    ) -> Lresult {
        match message {
            WM_PAINT => {
                let mut paint: PaintStruct = unsafe { std::mem::zeroed() };
                let hdc = unsafe { BeginPaint(hwnd, &mut paint) };
                if let Some(state) = unsafe { state_from(hwnd) } {
                    let mut area: Rect = unsafe { std::mem::zeroed() };
                    unsafe {
                        GetClientRect(hwnd, &mut area);
                    }
                    let bitmap = info(state);
                    unsafe {
                        StretchDIBits(
                            hdc,
                            0,
                            0,
                            area.right,
                            area.bottom,
                            0,
                            0,
                            state.width as i32,
                            state.height as i32,
                            state.bgr_pixels.as_ptr().cast(),
                            &bitmap,
                            DIB_RGB_COLORS,
                            SRCCOPY,
                        );
                    }
                }
                unsafe {
                    EndPaint(hwnd, &paint);
                }
                0
            }
            WM_KEYDOWN | WM_MOUSEWHEEL => {
                if let Some(state) = unsafe { state_from(hwnd) } {
                    let mut changed = true;
                    if message == WM_MOUSEWHEEL {
                        let delta = ((w_param >> 16) as u16 as i16) as f32;
                        state.camera.distance =
                            (state.camera.distance - delta.signum() * 2.0).clamp(8.0, 60.0);
                    } else {
                        match w_param {
                            VK_ESCAPE => {
                                unsafe {
                                    DestroyWindow(hwnd);
                                }
                                return 0;
                            }
                            VK_LEFT | 65 => state.camera.yaw -= 10.0,
                            VK_RIGHT | 68 => state.camera.yaw += 10.0,
                            VK_UP => {
                                state.camera.pitch = (state.camera.pitch + 5.0).clamp(-5.0, 75.0)
                            }
                            VK_DOWN => {
                                state.camera.pitch = (state.camera.pitch - 5.0).clamp(-5.0, 75.0)
                            }
                            87 | 107 | 187 => {
                                state.camera.distance = (state.camera.distance - 2.0).max(8.0)
                            }
                            83 | 109 | 189 => {
                                state.camera.distance = (state.camera.distance + 2.0).min(60.0)
                            }
                            _ => changed = false,
                        }
                    }
                    if changed {
                        unsafe {
                            update_title(hwnd, state);
                        }
                        render_state(state);
                        unsafe {
                            InvalidateRect(hwnd, std::ptr::null(), 0);
                        }
                    }
                }
                0
            }
            WM_DESTROY => {
                unsafe {
                    PostQuitMessage(0);
                }
                0
            }
            _ => unsafe { DefWindowProcW(hwnd, message, w_param, _l_param) },
        }
    }

    pub fn run(scene: Scene, camera: Camera, width: u32, height: u32) {
        unsafe {
            let instance = GetModuleHandleW(std::ptr::null());
            let class_name = wide("RustVoxelRaytracer");
            let class = WndClassEx {
                size: std::mem::size_of::<WndClassEx>() as u32,
                style: 0,
                proc: Some(wnd_proc),
                class_extra: 0,
                window_extra: 0,
                instance,
                icon: std::ptr::null_mut(),
                cursor: std::ptr::null_mut(),
                background: std::ptr::null_mut(),
                menu_name: std::ptr::null(),
                class_name: class_name.as_ptr(),
                icon_small: std::ptr::null_mut(),
            };
            if RegisterClassExW(&class) == 0 {
                panic!("Could not register the native Windows window");
            }
            let title = wide("Diorama - preparando render...");
            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                title.as_ptr(),
                WS_OVERLAPPEDWINDOW,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                (width * 2) as i32,
                (height * 2 + 60) as i32,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                instance,
                std::ptr::null_mut(),
            );
            if hwnd.is_null() {
                panic!("Could not create the native Windows window");
            }
            let mut boxed = Box::new(RenderState {
                scene,
                camera,
                width,
                height,
                bgr_pixels: Vec::new(),
            });
            render_state(&mut boxed);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(boxed) as isize);
            if let Some(state) = state_from(hwnd) {
                update_title(hwnd, state);
            }
            ShowWindow(hwnd, SW_SHOW);
            UpdateWindow(hwnd);
            let mut message: Msg = std::mem::zeroed();
            while GetMessageW(&mut message, std::ptr::null_mut(), 0, 0) > 0 {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
            let pointer = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut RenderState;
            if !pointer.is_null() {
                drop(Box::from_raw(pointer));
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        usage();
        return;
    }
    let width = parse_arg(&args, "--width", 480.0).max(16.) as u32;
    let height = parse_arg(&args, "--height", 320.0).max(16.) as u32;
    let camera = Camera {
        target: Vec3::new(0., 1., 0.),
        yaw: parse_arg(&args, "--yaw", 35.),
        pitch: parse_arg(&args, "--pitch", 22.),
        distance: parse_arg(&args, "--distance", 26.),
        fov: 52.,
    };
    let scene = build_scene();
    if let Some(output) = args
        .iter()
        .position(|a| a == "--output")
        .and_then(|i| args.get(i + 1))
    {
        let pixels = render_pixels(&scene, &camera, width, height);
        save_image(output, width, height, &pixels);
        println!("Saved {output}");
    } else {
        #[cfg(target_os = "windows")]
        native_window::run(scene, camera, width, height);
        #[cfg(not(target_os = "windows"))]
        eprintln!("Use --output file.bmp outside Windows.");
    }
}
