use crate::materials::default_materials;
use crate::scene::Scene;

pub struct SceneBuilder;

impl SceneBuilder {
    pub fn build() -> Scene {
        const GARDEN_HALF_SIZE: i32 = 32;
        const PATH: usize = 11;
        const TREE_TRUNK: usize = 10;
        let mut scene = Scene::new(default_materials());

        // -32..31 gives the scene an exact 64x64 footprint while keeping the
        // original composition centered around the origin.
        for x in -GARDEN_HALF_SIZE..GARDEN_HALF_SIZE {
            for z in -GARDEN_HALF_SIZE..GARDEN_HALF_SIZE {
                scene.add_cube(x as f32, -2., z as f32, 2);
                let river = (-22..=-20).contains(&x) && (2..=31).contains(&z);
                let main_path = (x == -5 || x == -4) && (1..=31).contains(&z);
                let path = main_path;
                scene.add_cube(
                    x as f32,
                    -1.,
                    z as f32,
                    if river {
                        5
                    } else if path {
                        PATH
                    } else {
                        0
                    },
                );
            }
        }

        Self::build_vegeta_house(&mut scene);
        Self::build_right_balcony(&mut scene);
        Self::build_left_river_bridge(&mut scene);

        for y in 0..4 {
            for x in 18..=23 {
                for z in 0..=3 {
                    if x == 18 || x == 23 || z == 0 || z == 3 {
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
        for x in 18..=23 {
            for z in 0..=3 {
                scene.add_cube(x as f32, 4., z as f32, 6);
            }
        }

        for &(tree_x, tree_z) in &[(-18, 22), (18, -8), (22, -14)] {
            for y in 0..4 {
                scene.add_cube(tree_x as f32, y as f32, tree_z as f32, TREE_TRUNK);
            }
            for x in tree_x - 1..=tree_x + 1 {
                for y in 3..=5 {
                    for z in tree_z - 1..=tree_z + 1 {
                        if !(x == tree_x && y == 3 && z == tree_z) {
                            scene.add_cube(x as f32, y as f32, z as f32, 4);
                        }
                    }
                }
            }
            scene.add_cube(tree_x as f32, 6., tree_z as f32, 4);
        }

        for &(x, z) in &[
            (-13, 2),
            (-13, 8),
            (9, 5),
            (15, 10),
            (-1, 14),
            (20, 4),
            (-18, -6),
            (12, -20),
        ] {
            scene.add_cube(x as f32, 0., z as f32, 2);
        }
        scene.rebuild_bvh();
        scene
    }

    /// Builds the modern two-wing facade inspired by Vegeta777's Karmaland 4 house.
    fn build_vegeta_house(scene: &mut Scene) {
        const CONCRETE: usize = 7;
        const TERRACOTTA: usize = 8;
        const PURPLE_GLASS: usize = 9;
        const DOOR_LOWER: usize = 12;
        const DOOR_UPPER: usize = 13;

        let wings = [(-19, -7), (-2, 10)];

        // The two wings have a solid body so the house also reads correctly from
        // the sides. The front windows are placed slightly forward afterward.
        for &(min_x, max_x) in &wings {
            for x in min_x..=max_x {
                for y in 0..5 {
                    for z in -9..=0 {
                        scene.add_cube(x as f32, y as f32, z as f32, CONCRETE);
                    }
                }
            }

            // Lower windows remain broad.
            for x in min_x + 1..max_x {
                scene.add_cube(x as f32, 1., 0.08, PURPLE_GLASS);
            }

            // The second level has exactly five slim, tall panels per wing,
            // matching the vertical rhythm of the reference facade.
            for x in [min_x + 2, min_x + 4, min_x + 6, min_x + 8, min_x + 10] {
                scene.add_box(x as f32 + 0.22, 3., 1.08, 0.56, 1.70, 0.10, PURPLE_GLASS);
            }

            // The lower band separates the floors; the upper roof edge now
            // frames the tall windows without cutting through them.
            for &y in &[0, 2] {
                for x in min_x..=max_x {
                    scene.add_cube(x as f32, y as f32, 0.14, TERRACOTTA);
                }
            }
        }

        // Taller entrance block in the center of the composition.
        for x in -6..=-3 {
            for y in 0..8 {
                for z in -9..=0 {
                    scene.add_cube(x as f32, y as f32, z as f32, CONCRETE);
                }
            }
        }

        // The central window is deliberately narrow and vertical, with the dark
        // wood door directly below it.
        for x in -5..=-4 {
            for y in 3..=6 {
                scene.add_cube(x as f32, y as f32, 1.08, PURPLE_GLASS);
            }
        }
        for x in -5..=-4 {
            scene.add_cube(x as f32, 0., 1.08, DOOR_LOWER);
            scene.add_cube(x as f32, 1., 1.08, DOOR_UPPER);
        }

        // Central frame, entrance lintel and raised roof cap.
        for &x in &[-6, -3] {
            for y in 0..8 {
                scene.add_cube(x as f32, y as f32, 1.08, CONCRETE);
            }
        }
        for x in -6..=-3 {
            // The lintel sits above both door halves, leaving y=1 clear for
            // puerta_superior.png.
            scene.add_cube(x as f32, 2., 1.10, TERRACOTTA);
            scene.add_cube(x as f32, 7., 1.08, CONCRETE);
        }
        Self::build_roof(scene);
        Self::build_back_facade(scene);
    }

    /// Builds the broad flat roof, recessed skylight and stepped upper section
    /// shown in the reference. The one-block overhang makes the roof read as a
    /// single architectural element from the front and rear.
    fn build_roof(scene: &mut Scene) {
        const CONCRETE: usize = 7;
        const PURPLE_GLASS: usize = 9;

        let skylight = |x: i32, z: i32| (-13..=-10).contains(&x) && (-6..=-3).contains(&z);

        for &(min_x, max_x) in &[(-20, -7), (-2, 11)] {
            for x in min_x..=max_x {
                for z in -10..=1 {
                    scene.add_cube(
                        x as f32,
                        5.,
                        z as f32,
                        if skylight(x, z) {
                            PURPLE_GLASS
                        } else {
                            CONCRETE
                        },
                    );
                }
            }
        }

        // The entrance block rises above the main roof as a central terrace.
        for x in -7..=-2 {
            for z in -10..=1 {
                scene.add_cube(x as f32, 7., z as f32, CONCRETE);
            }
        }

        // Raised cream border around the purple skylight.
        for x in -14..=-9 {
            scene.add_cube(x as f32, 6., -7., CONCRETE);
            scene.add_cube(x as f32, 6., -2., CONCRETE);
        }
        for z in -6..=-3 {
            scene.add_cube(-14., 6., z as f32, CONCRETE);
            scene.add_cube(-9., 6., z as f32, CONCRETE);
        }

        // Four progressively smaller terraces form the stair-stepped roof face.
        for level in 0..4 {
            let min_x = -1 + level;
            let max_x = 10 - level;
            let min_z = -8 + level;
            let max_z = -level;
            for x in min_x..=max_x {
                for z in min_z..=max_z {
                    scene.add_cube(x as f32, (6 + level) as f32, z as f32, CONCRETE);
                }
            }
        }
    }

    /// Builds the quieter rear facade: a long cream wall, a few purple windows,
    /// a continuous floor ledge and the stepped roof silhouette.
    fn build_back_facade(scene: &mut Scene) {
        const CONCRETE: usize = 7;
        const PURPLE_GLASS: usize = 9;
        const BACK_Z: f32 = -9.16;

        // Continuous ledges make the rear read as one large modern volume.
        for x in -19..=10 {
            scene.add_cube(x as f32, 2., BACK_Z, CONCRETE);
        }

        // Long, low horizontal window on the lower level.
        for x in -13..=-9 {
            scene.add_cube(x as f32, 1., BACK_Z, PURPLE_GLASS);
        }

        // Two separated upper openings reproduce the asymmetrical rear shown in
        // the reference image while leaving most of the wall intentionally plain.
        for x in -7..=-5 {
            scene.add_cube(x as f32, 3., BACK_Z, PURPLE_GLASS);
        }
        for x in 0..=2 {
            scene.add_cube(x as f32, 3., BACK_Z, PURPLE_GLASS);
        }
    }

    /// Adds the open right-side balcony visible in the reference: a light
    /// terrace, cream frame and thin wooden railing.
    fn build_right_balcony(scene: &mut Scene) {
        const CONCRETE: usize = 7;
        const WOOD: usize = 3;

        let min_x = 11;
        let max_x = 15;
        let back_z = -2.;
        let front_z = 4.;

        // Open lower level: four slimmer columns carry the terrace.
        for &x in &[min_x as f32 + 0.15, max_x as f32 + 0.15] {
            for &z in &[back_z + 0.15, front_z - 0.85] {
                scene.add_box(x, 0., z, 0.70, 2.0, 0.70, CONCRETE);
            }
        }

        // Solid light floor; the purple blocks are replaced by a thin wooden
        // railing so the balcony reads as an open, usable terrace.
        scene.add_box(
            min_x as f32,
            2.,
            back_z,
            (max_x - min_x + 1) as f32,
            0.25,
            front_z - back_z,
            CONCRETE,
        );

        // Thin wooden front rails and evenly spaced balusters.
        scene.add_box(min_x as f32, 3.05, front_z, 5., 0.14, 0.14, WOOD);
        scene.add_box(min_x as f32, 2.55, front_z, 5., 0.10, 0.10, WOOD);
        for x in min_x..=max_x {
            scene.add_box(x as f32 + 0.44, 2.25, front_z, 0.12, 0.85, 0.12, WOOD);
        }

        // Complete the rear edge with the same thin wooden railing so the
        // balcony is enclosed on every exposed side.
        scene.add_box(min_x as f32, 3.05, back_z, 5., 0.14, 0.14, WOOD);
        scene.add_box(min_x as f32, 2.55, back_z, 5., 0.10, 0.10, WOOD);
        for x in min_x..=max_x {
            scene.add_box(x as f32 + 0.44, 2.25, back_z, 0.12, 0.85, 0.12, WOOD);
        }

        // Close the right side with the same thinner wooden treatment.
        scene.add_box(max_x as f32 + 0.86, 3.05, back_z, 0.14, 0.14, 6., WOOD);
        for z in -2..=3 {
            scene.add_box(
                max_x as f32 + 0.86,
                2.25,
                z as f32 + 0.44,
                0.12,
                0.85,
                0.12,
                WOOD,
            );
        }

        // The marked side also receives a complete railing, filling the
        // remaining open edge while keeping the entrance toward the house
        // visually light and accessible.
        scene.add_box(min_x as f32, 3.05, back_z, 0.14, 0.14, 6., WOOD);
        scene.add_box(min_x as f32, 2.55, back_z, 0.10, 0.10, 6., WOOD);
        for z in -2..=3 {
            scene.add_box(min_x as f32, 2.25, z as f32 + 0.44, 0.12, 0.85, 0.12, WOOD);
        }
    }

    /// Connects the left garden path across the narrow river with a compact
    /// wooden bridge and handrails.
    fn build_left_river_bridge(scene: &mut Scene) {
        const WOOD: usize = 3;

        let bridge_x = -24.;
        let bridge_z = 9.;
        let bridge_width = 6.;
        let bridge_depth = 3.;

        // Raised planks span the water while ending on the two dirt paths.
        scene.add_box(
            bridge_x,
            0.05,
            bridge_z,
            bridge_width,
            0.22,
            bridge_depth,
            WOOD,
        );

        // Two slim rails follow the bridge length, with three posts each.
        for &z in &[bridge_z, bridge_z + bridge_depth - 0.12] {
            scene.add_box(bridge_x, 1.00, z, bridge_width, 0.12, 0.12, WOOD);
            scene.add_box(bridge_x, 0.55, z, bridge_width, 0.10, 0.10, WOOD);
            for &x in &[bridge_x + 0.10, bridge_x + 2.95, bridge_x + 5.78] {
                scene.add_box(x, 0.27, z, 0.12, 0.85, 0.12, WOOD);
            }
        }
    }
}
