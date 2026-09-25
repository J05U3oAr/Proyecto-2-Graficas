use crate::geometry::Vec3;

#[derive(Clone, Copy)]
pub enum Texture {
    Grass,
    Dirt,
    Stone,
    Wood,
    Leaves,
    Water,
    Glass,
}

#[derive(Clone, Copy)]
pub struct Material {
    pub texture: Texture,
    pub albedo: Vec3,
    pub specular: f32,
    pub shininess: f32,
    pub reflectivity: f32,
    pub transparency: f32,
    pub ior: f32,
}

pub fn default_materials() -> Vec<Material> {
    let white = Vec3::new(1., 1., 1.);
    vec![
        Material {
            texture: Texture::Grass,
            albedo: white,
            specular: 0.05,
            shininess: 12.,
            reflectivity: 0.,
            transparency: 0.,
            ior: 1.,
        },
        Material {
            texture: Texture::Dirt,
            albedo: white,
            specular: 0.02,
            shininess: 8.,
            reflectivity: 0.,
            transparency: 0.,
            ior: 1.,
        },
        Material {
            texture: Texture::Stone,
            albedo: white,
            specular: 0.16,
            shininess: 28.,
            reflectivity: 0.06,
            transparency: 0.,
            ior: 1.,
        },
        Material {
            texture: Texture::Wood,
            albedo: white,
            specular: 0.08,
            shininess: 16.,
            reflectivity: 0.,
            transparency: 0.,
            ior: 1.,
        },
        Material {
            texture: Texture::Leaves,
            albedo: white,
            specular: 0.03,
            shininess: 8.,
            reflectivity: 0.,
            transparency: 0.,
            ior: 1.,
        },
        Material {
            texture: Texture::Water,
            albedo: white,
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
    ]
}

fn fract(value: f32) -> f32 {
    value - value.floor()
}
fn hash(x: f32, y: f32) -> f32 {
    fract((x * 127.1 + y * 311.7).sin() * 43758.545)
}
fn checker(u: f32, v: f32, scale: f32) -> bool {
    ((u * scale).floor() as i32 + (v * scale).floor() as i32) & 1 == 0
}

pub fn sample_texture(kind: Texture, u: f32, v: f32) -> Vec3 {
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
