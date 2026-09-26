use crate::geometry::Vec3;
use std::sync::OnceLock;

#[derive(Clone, Copy)]
pub enum AssetTexture {
    Water,
    Path,
    Quartz,
    QuartzBrick,
    Wood,
    DoorLower,
    DoorUpper,
}

#[derive(Clone, Copy)]
pub enum Texture {
    Grass,
    Dirt,
    Stone,
    Leaves,
    Glass,
    PurpleGlass,
    TreeTrunk,
    Asset(AssetTexture),
}

struct LoadedTexture {
    width: u32,
    height: u32,
    pixels: Vec<Vec3>,
}

static ASSET_TEXTURES: OnceLock<[LoadedTexture; 7]> = OnceLock::new();

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
            texture: Texture::Asset(AssetTexture::Wood),
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
            texture: Texture::Asset(AssetTexture::Water),
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
        Material {
            texture: Texture::Asset(AssetTexture::Quartz),
            albedo: Vec3::new(1., 1., 1.),
            specular: 0.12,
            shininess: 24.,
            reflectivity: 0.02,
            transparency: 0.,
            ior: 1.,
        },
        Material {
            texture: Texture::Asset(AssetTexture::QuartzBrick),
            albedo: Vec3::new(1., 1., 1.),
            specular: 0.16,
            shininess: 28.,
            reflectivity: 0.03,
            transparency: 0.,
            ior: 1.,
        },
        Material {
            texture: Texture::PurpleGlass,
            albedo: Vec3::new(0.82, 0.62, 0.94),
            specular: 0.65,
            shininess: 85.,
            reflectivity: 0.08,
            transparency: 0.28,
            ior: 1.52,
        },
        Material {
            texture: Texture::TreeTrunk,
            albedo: white,
            specular: 0.08,
            shininess: 16.,
            reflectivity: 0.,
            transparency: 0.,
            ior: 1.,
        },
        Material {
            texture: Texture::Asset(AssetTexture::Path),
            albedo: white,
            specular: 0.02,
            shininess: 8.,
            reflectivity: 0.,
            transparency: 0.,
            ior: 1.,
        },
        Material {
            texture: Texture::Asset(AssetTexture::DoorLower),
            albedo: white,
            specular: 0.08,
            shininess: 16.,
            reflectivity: 0.,
            transparency: 0.,
            ior: 1.,
        },
        Material {
            texture: Texture::Asset(AssetTexture::DoorUpper),
            albedo: white,
            specular: 0.08,
            shininess: 16.,
            reflectivity: 0.,
            transparency: 0.,
            ior: 1.,
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
            // High-frequency cells create tiny grass particles without adding
            // extra geometry to the scene. Most cells remain unchanged, while
            // a few receive a brighter green blade/fleck variation.
            let particle_u = (u * 48.).floor();
            let particle_v = (v * 48.).floor();
            let particle_seed = hash(particle_u, particle_v);
            let particle = ((particle_seed - 0.86) / 0.14).clamp(0., 1.);
            let particle_tint = Vec3::new(0.08, 0.18, 0.035) * particle;
            Vec3::new(0.20 + blades, 0.48 + grain * 0.16, 0.12 + blades * 0.3) + particle_tint
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
        Texture::TreeTrunk => {
            let rings = ((u * 17. + (v * 8.).sin() * 1.5).sin() * 0.5 + 0.5) * 0.20;
            Vec3::new(0.27 + rings, 0.105 + rings * 0.48, 0.028 + rings * 0.20)
        }
        Texture::Leaves => Vec3::new(
            0.055 + grain * 0.07,
            0.23 + grain * 0.20,
            0.045 + grain * 0.06,
        ),
        Texture::Glass => Vec3::new(0.66, 0.89, 0.91),
        Texture::PurpleGlass => Vec3::new(
            0.46 + grain * 0.06,
            0.14 + grain * 0.035,
            0.62 + grain * 0.08,
        ),
        Texture::Asset(asset) => sample_asset(asset, u, v),
    }
}

fn sample_asset(asset: AssetTexture, u: f32, v: f32) -> Vec3 {
    let textures = ASSET_TEXTURES.get_or_init(|| {
        [
            load_asset(include_bytes!("../assets/Agua.png")),
            load_asset(include_bytes!("../assets/camino.png")),
            load_asset(include_bytes!("../assets/cuarzo.png")),
            load_asset(include_bytes!("../assets/ladrillo_cuarzo.png")),
            load_asset(include_bytes!("../assets/madera.png")),
            load_asset(include_bytes!("../assets/puerta_inferior.png")),
            load_asset(include_bytes!("../assets/puerta_superior.png")),
        ]
    });
    let texture = &textures[asset_index(asset)];
    let x = (fract(u) * texture.width as f32) as u32 % texture.width;
    let y = ((1. - fract(v)) * texture.height as f32) as u32 % texture.height;
    texture.pixels[(y * texture.width + x) as usize]
}

fn asset_index(asset: AssetTexture) -> usize {
    match asset {
        AssetTexture::Water => 0,
        AssetTexture::Path => 1,
        AssetTexture::Quartz => 2,
        AssetTexture::QuartzBrick => 3,
        AssetTexture::Wood => 4,
        AssetTexture::DoorLower => 5,
        AssetTexture::DoorUpper => 6,
    }
}

fn load_asset(bytes: &[u8]) -> LoadedTexture {
    let image = image::load_from_memory(bytes)
        .expect("No se pudo cargar una textura desde la carpeta assets")
        .to_rgba8();
    let (width, height) = image.dimensions();
    let pixels = image
        .pixels()
        .map(|pixel| {
            Vec3::new(
                srgb_to_linear(pixel[0]),
                srgb_to_linear(pixel[1]),
                srgb_to_linear(pixel[2]),
            )
        })
        .collect();
    LoadedTexture {
        width,
        height,
        pixels,
    }
}

fn srgb_to_linear(value: u8) -> f32 {
    let value = value as f32 / 255.;
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}
