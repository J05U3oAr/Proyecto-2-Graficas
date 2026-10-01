//! Reference house: +Z is the pool/front, -X the panoramic left wing.
//! All geometry is axis-aligned; rooms are empty shells, not filled volumes.
use crate::geometry::Vec3;
use crate::materials::{default_materials, Material, Texture};
use crate::scene::Scene;

const WHITE: usize = 7;
const GLASS: usize = 6;
const METAL: usize = 17;
const WOOD: usize = 3;

pub fn materials() -> Vec<Material> {
    let mut materials = default_materials();
    materials[WHITE].texture = Texture::Solid;
    materials[WHITE].albedo = Vec3::new(0.94, 0.93, 0.89);
    materials[GLASS].texture = Texture::Solid;
    materials[GLASS].albedo = Vec3::new(0.94, 0.98, 1.);
    materials[GLASS].transparency = 0.86;
    materials[GLASS].reflectivity = 0.07;
    materials.push(Material {
        texture: Texture::Solid,
        albedo: Vec3::new(0.15, 0.18, 0.20),
        specular: 0.45,
        shininess: 80.,
        reflectivity: 0.12,
        transparency: 0.,
        ior: 1.,
    });
    materials
}

struct HouseModel<'a> {
    scene: &'a mut Scene,
    scale: f32,
    origin: [f32; 3],
}

impl HouseModel<'_> {
    fn block(&mut self, min: [f32; 3], size: [f32; 3], material: usize) {
        self.scene.add_box(
            self.origin[0] + min[0] * self.scale,
            self.origin[1] + min[1] * self.scale,
            self.origin[2] + min[2] * self.scale,
            size[0] * self.scale,
            size[1] * self.scale,
            size[2] * self.scale,
            material,
        );
    }

    // Single panes with fine edge profiles; no opaque backing or voxel grid.
    fn front_glass(&mut self, x: f32, y: f32, z: f32, w: f32, h: f32) {
        self.block([x, y, z], [w, h, 0.10], GLASS);
        for px in [x, x + w - 0.045] {
            self.block([px, y, z + 0.10], [0.045, h, 0.045], METAL);
        }
        for py in [y, y + h - 0.045] {
            self.block([x, py, z + 0.10], [w, 0.045, 0.045], METAL);
        }
    }

    fn side_glass(&mut self, x: f32, y: f32, z: f32, d: f32, h: f32) {
        self.block([x, y, z], [0.10, h, d], GLASS);
        for pz in [z, z + d - 0.045] {
            self.block([x - 0.045, y, pz], [0.045, h, 0.045], METAL);
        }
        for py in [y, y + h - 0.045] {
            self.block([x - 0.045, py, z], [0.045, 0.045, d], METAL);
        }
    }

    // A framed glass door on an X-facing facade. It sits slightly in front of
    // the continuous glazing, so an exterior stair has a clear destination
    // without breaking the large glass pane behind it.
    fn side_entry_door(&mut self, x: f32, y: f32, z: f32, d: f32, h: f32) {
        self.block([x, y, z], [0.12, h, d], GLASS);

        for pz in [z, z + d - 0.10] {
            self.block([x - 0.04, y, pz], [0.16, h, 0.10], METAL);
        }
        for py in [y, y + h - 0.10] {
            self.block([x - 0.04, py, z], [0.16, 0.10, d], METAL);
        }

        // Slim pull handle facing the exterior stair.
        self.block(
            [x - 0.12, y + h * 0.36, z + d - 0.32],
            [0.14, h * 0.30, 0.09],
            METAL,
        );
    }

    fn landscape(&mut self) {
        // White rectangular approach and the shallow L-shaped pool beneath
        // the left pilotis. The raised right basin shares its water level.
        self.block([-16., 0., -17.], [32., 0.45, 35.], WHITE);
        self.block([-12.5, 0.46, 3.], [24.5, 0.18, 11.], 14);
        self.block([-12.5, 0.64, 3.], [24.5, 0.12, 11.], 5);
        self.block([-13.5, 0.45, 2.], [1., 0.9, 13.], WHITE);
        self.block([-13.5, 0.45, 14.], [8., 0.9, 1.], WHITE);
        self.block([-3., 0.45, 12.], [16., 1.15, 1.3], WHITE);
        self.block([11.7, 0.45, 3.], [1.3, 1.15, 9.], WHITE);
        self.block([6., 0.45, 3.], [5.7, 1.15, 1.], WHITE);
        self.block([-16., 0.45, 17.], [11., 0.8, 1.], WHITE);
        self.block([-2., 0.45, 17.], [18., 0.8, 1.], WHITE);
        self.block([-16., 0.45, 3.], [0.8, 0.8, 14.], WHITE);
        self.block([15.2, 0.45, 3.], [0.8, 0.8, 14.], WHITE);
        for x in -3..14 {
            self.block([x as f32, 0.45, 15.], [1., 1.4, 1.2], 4);
        }
        for z in 5..15 {
            self.block([13., 0.45, z as f32], [1., 1.4, 1.], 4);
        }
        self.block([-5., 0.46, 14.], [3., 0.20, 9.], WOOD);
        for step in 0..6 {
            self.block(
                [-5., 0.65, 13. - step as f32 * 0.65],
                [3., 0.25 + step as f32 * 0.30, 0.65],
                WOOD,
            );
        }
        // Bridge over the pool leads to the recessed central entrance.
        self.block([-5., 2.40, 3.], [5., 0.28, 6.75], WHITE);
    }

    fn left_wing(&mut self) {
        // Long horizontal slabs wrap the front, side and rear of the salon.
        self.block([-13., 6.8, -16.], [11., 1.0, 21.], WHITE);
        self.block([-14., 14.6, -16.], [12., 1.0, 22.], WHITE);
        self.block([-13., 0.45, -16.], [11., 0.7, 16.], WHITE);
        for (x, z) in [(-11.7, 3.5), (-12.7, -15.5), (-3., -15.5)] {
            self.block([x, 1.15, z], [0.9, 5.65, 0.9], WHITE);
        }
        self.front_glass(-11.8, 7.8, 3.5, 8.8, 6.8);
        self.side_glass(-11.9, 7.8, -15., 18.5, 6.8);
        self.front_glass(-11.8, 7.8, -15.1, 8.8, 6.8);
        // Right jamb of the front C-shaped frame; the outside glass corner
        // remains exposed, as in the first two reference images.
        self.block([-3., 6.8, 3.5], [1., 8.8, 1.5], WHITE);
        self.block([-13., 7.8, -16.], [1.1, 6.8, 1.], WHITE);
        self.front_glass(-11.8, 1.15, -0.1, 8.8, 5.65);
        self.side_glass(-11.9, 1.15, -15., 14.9, 5.65);
        // The smaller recessed roof room opens onto the cantilevered terrace.
        self.block([-11., 15.6, -16.], [1., 5.2, 1.2], WHITE);
        self.block([-11., 15.6, -1.1], [1., 5.2, 1.1], WHITE);
        self.block([-11., 15.6, -14.8], [1., 0.7, 13.7], WHITE);
        self.block([-11., 20.8, -16.], [9., 0.9, 17.], WHITE);
        self.block([-3., 15.6, -16.], [1., 5.2, 17.], WHITE);
        self.front_glass(-10., 15.6, -0.1, 7., 5.2);
        self.side_glass(-10.8, 16.3, -14.8, 13.7, 4.5);
        self.front_glass(-10., 15.6, -15.1, 7., 5.2);
        self.front_glass(-11.8, 15.6, 3.8, 8.8, 1.25);
        self.side_glass(-11.9, 15.6, -1., 4.8, 1.25);
        // Exterior staircase rises from pool level to the first-floor slab.
        for step in 0..16 {
            let rise = (step + 1) as f32 * (7.35 / 16.);
            let z = 9. - step as f32 * 0.62;
            self.block([-15., 0.45, z], [2.3, rise, 0.62], WOOD);
            self.block([-15.5, 0.45, z], [0.5, rise + 0.45, 0.62], WHITE);
        }
        // A compact balcony extends the final stair tread into a usable entry
        // landing. Glass rails protect its exposed edges and leave the stair
        // side open for access.
        self.block([-15.5, 7.55, -2.55], [3.55, 0.25, 3.20], WHITE);
        self.side_glass(-15.60, 7.8, -2.55, 3.20, 1.25);
        self.front_glass(-15.5, 7.8, -2.65, 3.55, 1.25);

        // This balcony is reached by the exterior staircase above. The framed
        // glass door makes the circulation route visibly lead into the salon.
        self.side_entry_door(-12.20, 7.8, -2.25, 2.15, 5.20);
    }

    fn atrium(&mut self) {
        // Continuous tall transparent spine, flanked by white fins.
        for x in [-2., 2.3] {
            self.block([x, 0.45, -16.], [0.8, 24.55, 0.8], WHITE);
            self.block([x, 0.45, 3.2], [0.8, 24.55, 0.8], WHITE);
            // Above the salon the tall fins become solid walls. Below it,
            // openings connect both wings so the side view crosses the house.
            self.block([x, 15.6, -15.2], [0.8, 9.4, 18.4], WHITE);
        }
        self.front_glass(-1.2, 0.75, 3.8, 3.5, 23.7);
        self.front_glass(-1.2, 0.75, -16., 3.5, 23.7);
        self.block([-2., 24.4, -16.], [5.1, 0.6, 0.8], WHITE);
        self.block([-1.2, 24.35, -15.2], [3.5, 0.12, 19.], GLASS);
        for y in [7.5, 14.8, 20.8] {
            // Recess the bridges so the front glass reads as one tall opening.
            self.block([-1.2, y, -13.], [3.5, 0.40, 13.], WHITE);
            self.block([-1.2, y, 3.7], [3.5, 0.14, 0.12], METAL);
        }
        for y in [21.8, 23.1, 24.4] {
            self.block([-1.2, y, 3.7], [3.5, 0.07, 0.08], METAL);
        }
    }

    fn right_wing(&mut self) {
        // A single tall front frame contains two levels and an inset balcony.
        self.block([3.1, 6.8, -15.2], [9.2, 1., 19.7], WHITE);
        self.block([12.3, 6.8, 1.6], [1., 1., 2.9], WHITE);
        self.block([3.1, 21., -16.], [10.2, 1.1, 20.5], WHITE);
        self.block([12.3, 7.8, 3.3], [1., 13.2, 1.2], WHITE);
        self.block([3.1, 7.8, 3.3], [0.65, 13.2, 1.2], WHITE);
        self.front_glass(3.75, 7.8, 3.3, 8.55, 6.8);
        self.front_glass(3.75, 15.3, 0.9, 8.55, 5.7);
        self.block([3.75, 14.6, -15.2], [8.55, 0.7, 18.3], WHITE);
        self.front_glass(3.75, 15.3, 3.05, 8.55, 1.30);
        self.side_glass(3.75, 15.3, 0.9, 2.15, 1.30);
        self.block([5.3, 7.8, -0.5], [1., 13.2, 1.1], WHITE);
        self.block([3.1, 0.45, -16.], [10.2, 0.70, 16.5], WHITE);
        self.front_glass(3.75, 1.15, 0.2, 8.55, 5.65);
        // Six tall slots on the right; their wall is a shell shared by all
        // floors, with a stepped parapet rather than separate window stickers.
        self.block([12.3, 1.15, -16.], [1., 19.85, 0.8], WHITE);
        for i in 0..6 {
            let z = -15.2 + i as f32 * 2.8;
            self.side_glass(12.65, 1.15, z, 1.1, 18.2);
            self.block([12.3, 1.15, z + 1.1], [1., 19.85, 1.7], WHITE);
        }
        self.block([12.3, 19.35, -15.2], [1., 1.65, 16.8], WHITE);
        self.block([12.3, 22.1, -13.5], [1., 0.9, 13.], WHITE);
        self.block([12.3, 23., -10.5], [1., 0.9, 7.], WHITE);
        // Rear's three narrow slots continue around the right corner.
        self.block([3.1, 1.15, -16.], [0.65, 19.85, 0.8], WHITE);
        for i in 0..3 {
            let x = 3.75 + i as f32 * 2.85;
            self.front_glass(x, 1.15, -15.9, 1.1, 18.2);
            self.block([x + 1.1, 1.15, -16.], [1.75, 19.85, 0.8], WHITE);
        }
        self.block([3.1, 19.35, -16.], [10.2, 1.65, 0.8], WHITE);
    }

    fn rear_and_interior(&mut self) {
        // Rear lower vertical rhythm under the two panoramic openings.
        for i in 0..4 {
            let x = -11.9 + i as f32 * 2.25;
            self.front_glass(x, 1.15, -15.9, 1., 5.65);
            self.block([x + 1., 1.15, -16.], [1.25, 5.65, 0.8], WHITE);
        }
        self.block([-13., 1.15, -16.], [1.1, 5.65, 0.8], WHITE);
        // Upper rear lintels project over the glass like the reference's E.
        self.block([-11., 20.8, -16.8], [9., 0.9, 0.8], WHITE);
        self.block([-13., 14.6, -16.8], [11., 1., 0.8], WHITE);
        self.block([-13., 6.8, -16.8], [11., 1., 0.8], WHITE);
        // Circulation visible through the side slots and the left salon.
        for (base, reverse) in [(1.15, false), (7.8, true)] {
            for step in 0..17 {
                let z = if reverse {
                    -3. - step as f32 * 0.60
                } else {
                    -13. + step as f32 * 0.60
                };
                self.block([9.5, base + step as f32 * 0.40, z], [2.4, 0.4, 0.65], WOOD);
            }
        }
        // Door frame behind the upper terrace and the lower entrance.
        for (x, y, z) in [(-7.6, 15.6, -3.), (-8., 1.15, -3.)] {
            self.front_glass(x, y, z, 2.3, 3.8);
            self.block([x + 2.05, y + 1.8, z + 0.15], [0.08, 0.35, 0.08], METAL);
        }
    }
}

/// Same geometry serves the full scene and a scaled menu miniature.
pub fn build(scene: &mut Scene, scale: f32, origin: [f32; 3]) {
    let mut model = HouseModel {
        scene,
        scale,
        origin,
    };
    model.landscape();
    model.left_wing();
    model.atrium();
    model.right_wing();
    model.rear_and_interior();
}
