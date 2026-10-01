//! Auropl: the reference's front faces +Z. Three wooden elevated rooms share
//! striped access shafts and a low U-shaped gallery. Hidden elevations follow
//! the same timber trim, recessed windows and flat roofs as the reference.
use crate::geometry::Vec3;
use crate::materials::{default_materials, Material, Texture};
use crate::scene::Scene;

const OAK: usize = 17;
const DARK_WOOD: usize = 18;
const RED: usize = 19;
const PINK: usize = 20;
const GOLD: usize = 21;
const IRON: usize = 22;
const IVORY: usize = 23;
const WINDOW: usize = 24;
const LAMP: usize = 25;

pub fn materials() -> Vec<Material> {
    let mut result = default_materials();
    result[5].reflectivity = 0.05;
    result[5].transparency = 0.18;
    for (texture, rgb, reflectivity) in [
        (Texture::WoodPlanks, [0.66, 0.43, 0.18], 0.0),
        (Texture::WoodPlanks, [0.30, 0.18, 0.065], 0.0),
        (Texture::Solid, [0.41, 0.13, 0.065], 0.0),
        (Texture::Solid, [0.59, 0.32, 0.40], 0.0),
        (Texture::Solid, [1.0, 0.77, 0.055], 0.12),
        (Texture::Solid, [0.028, 0.035, 0.045], 0.05),
        (Texture::Solid, [0.86, 0.89, 0.85], 0.08),
        (Texture::SpeckledGlass, [0.91, 0.92, 0.83], 0.02),
        (Texture::Solid, [0.95, 0.94, 0.72], 0.04),
    ] {
        result.push(Material {
            texture,
            albedo: Vec3::from(rgb),
            specular: 0.18,
            shininess: 48.,
            reflectivity,
            transparency: 0.,
            ior: 1.,
        });
    }
    result
}

struct Model<'a> {
    scene: &'a mut Scene,
    scale: f32,
    origin: [f32; 3],
}

impl Model<'_> {
    fn block(&mut self, p: [f32; 3], size: [f32; 3], material: usize) {
        self.scene.add_box(
            self.origin[0] + p[0] * self.scale,
            self.origin[1] + p[1] * self.scale,
            self.origin[2] + p[2] * self.scale,
            size[0] * self.scale,
            size[1] * self.scale,
            size[2] * self.scale,
            material,
        );
    }

    // Recessed horizontal glazing with terracotta end panels and timber trim.
    fn front_window(&mut self, x: f32, y: f32, z: f32, width: f32, rear: bool) {
        let trim_z = if rear { z - 0.10 } else { z + 0.10 };
        self.block([x, y, z], [width, 1.65, 0.12], WINDOW);
        for px in [x - 0.80, x + width] {
            self.block([px, y - 0.08, z], [0.80, 1.81, 0.16], RED);
        }
        for py in [y - 0.13, y + 1.65] {
            self.block([x - 0.10, py, trim_z], [width + 0.20, 0.13, 0.10], OAK);
        }
        for px in [x - 0.10, x + width] {
            self.block([px, y, trim_z], [0.10, 1.65, 0.10], OAK);
        }
    }

    fn room(&mut self, cx: f32, floor: f32, front: f32, width: f32, depth: f32) {
        let x = cx - width * 0.5;
        let back = front - depth;
        let h = 4.8;
        // Overhanging cornices and a dark inset underside make each pod read
        // as a deep inhabited volume when viewed from below, like the photo.
        for y in [floor, floor + h - 0.55] {
            self.block([x, y, back], [width, 0.55, depth], OAK);
        }
        self.block(
            [x + 0.65, floor - 0.12, back + 0.65],
            [width - 1.30, 0.12, depth - 1.30],
            DARK_WOOD,
        );
        let window_width = width * 0.57;
        let wx = cx - window_width * 0.5;
        // Front and rear have actual openings, filled by the patterned panes.
        for (z, rear) in [(back + 0.35, true), (front - 0.65, false)] {
            self.block(
                [x + 0.35, floor + 0.55, z],
                [width - 0.70, 0.85, 0.30],
                DARK_WOOD,
            );
            self.block(
                [x + 0.35, floor + 3.05, z],
                [width - 0.70, 1.20, 0.30],
                DARK_WOOD,
            );
            let jamb = (width - window_width) * 0.5 - 0.35;
            self.block([x + 0.35, floor + 1.40, z], [jamb, 1.65, 0.30], DARK_WOOD);
            self.block(
                [wx + window_width, floor + 1.40, z],
                [jamb, 1.65, 0.30],
                DARK_WOOD,
            );
            self.front_window(
                wx,
                floor + 1.40,
                if rear { z - 0.16 } else { z + 0.31 },
                window_width,
                rear,
            );
        }
        for px in [x, x + width - 0.50] {
            // Side windows use the same proportions and red end caps.
            self.block(
                [px, floor + 0.55, back + 0.35],
                [0.50, 0.85, depth - 0.70],
                OAK,
            );
            self.block(
                [px, floor + 3.05, back + 0.35],
                [0.50, 1.20, depth - 0.70],
                OAK,
            );
            for z in [back + 0.35, front - 1.75] {
                self.block([px, floor + 1.40, z], [0.50, 1.65, 1.40], OAK);
            }
            self.block(
                [px + 0.18, floor + 1.40, back + 1.75],
                [0.14, 1.65, depth - 3.50],
                WINDOW,
            );
            for z in [back + 1.75, front - 2.15] {
                self.block([px + 0.14, floor + 1.40, z], [0.22, 1.65, 0.40], RED);
            }
        }
        // Narrow raised borders wrap all four elevations.
        for z in [back, front - 0.22] {
            for y in [floor + 0.55, floor + h - 0.85] {
                self.block([x, y, z], [width, 0.30, 0.22], OAK);
            }
            for px in [x, x + width - 0.60] {
                self.block([px, floor + 0.55, z], [0.60, h - 1.10, 0.22], OAK);
            }
        }
    }

    fn shaft(&mut self, cx: f32, base: f32, top: f32, front: f32, center: bool) {
        let width = if center { 3.2 } else { 3.8 };
        let x = cx - width * 0.5;
        let depth = 2.8;
        let h = top - base;
        // Hollow timber shafts: matching stripes continue onto the rear.
        for px in [x, x + width - 0.55] {
            self.block([px, base, front - depth], [0.55, h, depth], OAK);
        }
        for (z, rear) in [(front - depth, true), (front - 0.18, false)] {
            self.block(
                [x + 0.55, base, z],
                [width - 1.10, h, 0.18],
                if center { RED } else { PINK },
            );
            for px in [x + 0.55, x + width - 0.85] {
                self.block(
                    [px, base, if rear { z - 0.08 } else { z + 0.18 }],
                    [0.30, h, 0.08],
                    RED,
                );
            }
            if center {
                self.block(
                    [cx - 0.33, base, if rear { z - 0.09 } else { z + 0.19 }],
                    [0.66, h, 0.08],
                    OAK,
                );
            }
        }
    }

    fn gallery(&mut self) {
        // Suspended low crosspiece completes the large U; the central shaft
        // continues below it to the entrance at ground level.
        for y in [8., 10.90] {
            self.block([-15.9, y, -2.2], [31.8, 0.45, 2.8], OAK);
        }
        for (z, stripe_z) in [(-2.2, -2.28), (0.42, 0.60)] {
            self.block([-15.35, 8.45, z], [30.7, 2.45, 0.18], PINK);
            self.block([-15.35, 8.45, stripe_z], [30.7, 0.28, 0.08], RED);
        }
        for cx in [-14., 14.] {
            self.shaft(cx, 8., 35.2, 0.60, false);
        }
        self.shaft(0., 4.4, 26.8, 2.5, true);
    }

    fn entrance(&mut self) {
        self.block([-3.1, 0., -1.1], [6.2, 0.25, 6.3], OAK);
        self.block([-2.8, 0.25, -1.0], [5.6, 4.15, 0.35], GOLD);
        for x in [-2.8, 2.30] {
            self.block([x, 0.25, -0.65], [0.50, 4.15, 5.15], GOLD);
        }
        self.block([-3., 4.15, -1.2], [6., 0.30, 5.90], OAK);
        // Gold blockwork around a pair of pale iron doors.
        for x in [-2.8, 1.3] {
            for row in 0..3 {
                self.block([x, 0.25 + row as f32 * 1.3, 4.15], [1.50, 1.24, 0.38], GOLD);
            }
        }
        self.block([-1.3, 3.15, 4.15], [2.6, 1., 0.38], GOLD);
        for x in [-1.25, 0.04] {
            self.block([x, 0.25, 4.22], [1.21, 2.90, 0.16], IVORY);
            for dx in [0.17, 0.66] {
                for y in [1.98, 2.51] {
                    self.block([x + dx, y, 4.39], [0.33, 0.36, 0.025], WINDOW);
                }
                self.block([x + dx, 0.50, 4.39], [0.33, 1.03, 0.035], 16);
            }
        }
        for x in [-0.20, 0.11] {
            self.block([x, 1.55, 4.43], [0.09, 0.25, 0.09], IRON);
        }
        // Small plaque above the doors and square lamps on the gold pilasters.
        self.block([-0.38, 3.47, 4.55], [0.76, 0.42, 0.06], DARK_WOOD);
        for x in [-2.25, 1.75] {
            self.block([x, 2.03, 4.55], [0.50, 0.63, 0.14], IVORY);
        }
    }

    fn lantern(&mut self, x: f32, y: f32, z: f32) {
        self.block([x - 0.25, y, z - 0.25], [0.50, 0.70, 0.50], LAMP);
        for px in [x - 0.29, x + 0.23] {
            for pz in [z - 0.29, z + 0.23] {
                self.block([px, y - 0.05, pz], [0.06, 0.80, 0.06], IRON);
            }
        }
        for py in [y - 0.12, y + 0.70] {
            self.block([x - 0.36, py, z - 0.36], [0.72, 0.12, 0.72], IRON);
        }
        self.block([x - 0.24, y + 0.82, z - 0.24], [0.48, 0.18, 0.48], IRON);
        self.block([x - 0.08, y + 1., z - 0.08], [0.16, 0.22, 0.16], IRON);
    }

    fn lamppost(&mut self, x: f32, z: f32) {
        self.block([x - 0.32, 0.12, z - 0.32], [0.64, 0.24, 0.64], IRON);
        self.block([x - 0.19, 0.36, z - 0.19], [0.38, 0.50, 0.38], IRON);
        self.block([x - 0.09, 0.86, z - 0.09], [0.18, 5.30, 0.18], IRON);
        self.lantern(x, 5.95, z);
        // Two smaller hanging lanterns, with squared scrollwork arms.
        for side in [-1., 1.] {
            let lx = x + side * 0.86;
            self.block([x.min(lx), 5.95, z - 0.06], [0.86, 0.12, 0.12], IRON);
            self.block([lx - 0.06, 5.62, z - 0.06], [0.12, 0.45, 0.12], IRON);
            self.lantern(lx, 4.55, z);
            self.block([x.min(lx), 4.05, z - 0.055], [0.86, 0.11, 0.11], IRON);
            self.block([lx - 0.055, 4.05, z - 0.055], [0.11, 0.50, 0.11], IRON);
        }
    }

    fn courtyard(&mut self) {
        self.block([-1.6, 0.015, 4.6], [3.2, 0.16, 12.8], OAK);
        self.block([-9.7, 0.015, 4.6], [19.4, 0.16, 1.8], OAK);
        for x in [-10., 1.65] {
            self.block([x, -0.35, 6.45], [8.35, 0.18, 8.7], 14);
            self.block([x, -0.17, 6.45], [8.35, 0.23, 8.7], 5);
            for px in [x - 0.12, x + 8.35] {
                self.block([px, 0.02, 6.33], [0.12, 0.10, 8.94], DARK_WOOD);
            }
            for z in [6.33, 15.15] {
                self.block([x, 0.02, z], [8.35, 0.10, 0.12], DARK_WOOD);
            }
        }
        for x in [-3.4, 3.4] {
            for z in [6., 13.8] {
                self.lamppost(x, z);
            }
        }
    }
}

/// The selector miniature and the full world share all architectural details.
pub fn build(scene: &mut Scene, scale: f32, origin: [f32; 3]) {
    let mut model = Model {
        scene,
        scale,
        origin,
    };
    model.courtyard();
    model.entrance();
    model.gallery();
    for x in [-14., 14.] {
        model.room(x, 35.2, 2.0, 17.5, 8.5);
    }
    model.room(0., 26.8, 4.0, 16., 8.5);
}

pub fn world() -> Scene {
    let mut scene = Scene::new(materials());
    // A quiet grassy island keeps the tall silhouette and front pools legible.
    for x in -31i32..=31 {
        for z in -25i32..=25 {
            if (x as f32 / 32.).powi(2) + (z as f32 / 26.).powi(2) > 1. {
                continue;
            }
            let pool = ((-10..=-2).contains(&x) || (1..=9).contains(&x)) && (6..=15).contains(&z);
            if !pool {
                scene.add_cube(x as f32, -1., z as f32, 0);
            }
            scene.add_cube(x as f32, -2., z as f32, 1);
            scene.add_cube(x as f32, -3., z as f32, 2);
        }
    }
    build(&mut scene, 1., [0., 0., 0.]);
    scene.rebuild_bvh();
    scene
}
