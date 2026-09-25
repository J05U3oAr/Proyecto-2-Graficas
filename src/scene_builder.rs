use crate::materials::default_materials;
use crate::scene::Scene;

pub struct SceneBuilder;

impl SceneBuilder {
    pub fn build() -> Scene {
        const GARDEN_HALF_SIZE: i32 = 32;
        let mut scene = Scene::new(default_materials());

        // -32..31 gives the scene an exact 64x64 footprint while keeping the
        // original composition centered around the origin.
        for x in -GARDEN_HALF_SIZE..GARDEN_HALF_SIZE {
            for z in -GARDEN_HALF_SIZE..GARDEN_HALF_SIZE {
                scene.add_cube(x as f32, -2., z as f32, 2);
                let lake = (-4..=4).contains(&x) && (2..=6).contains(&z);
                let path = (x == -5 || x == -4) && z < 2 && z > -GARDEN_HALF_SIZE;
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

        Self::build_vegeta_house(&mut scene);

        for y in 0..4 {
            for x in 9..=14 {
                for z in 0..=3 {
                    if x == 9 || x == 14 || z == 0 || z == 3 {
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
        for x in 9..=14 {
            for z in 0..=3 {
                scene.add_cube(x as f32, 4., z as f32, 6);
            }
        }

        for &(tree_x, tree_z) in &[(-20, 16), (14, -8), (22, -14)] {
            for y in 0..4 {
                scene.add_cube(tree_x as f32, y as f32, tree_z as f32, 3);
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
            (-20, -6),
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
        const DOOR: usize = 3;

        let wings = [(-15, -7), (-2, 6)];

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

            // Two large windows per level, separated by the facade columns.
            for x in min_x + 1..max_x {
                for &y in &[1, 3] {
                    scene.add_cube(x as f32, y as f32, 0.08, PURPLE_GLASS);
                }
            }

            // Terracotta bands reproduce the strong horizontal lines of the facade.
            for &y in &[0, 2, 4] {
                for x in min_x..=max_x {
                    scene.add_cube(x as f32, y as f32, 0.14, TERRACOTTA);
                }
            }

            // A shallow stepped roof gives the wings the layered silhouette seen
            // in the reference instead of leaving them as a simple flat box.
            for x in min_x..=max_x {
                for z in -9..=0 {
                    scene.add_cube(x as f32, 5., z as f32, CONCRETE);
                }
                for z in -8..=-2 {
                    scene.add_cube(x as f32, 6., z as f32, CONCRETE);
                }
            }
        }

        // Taller entrance block in the center of the composition.
        for x in -6..=-3 {
            for y in 0..7 {
                for z in -9..=0 {
                    scene.add_cube(x as f32, y as f32, z as f32, CONCRETE);
                }
            }
        }

        // The central window is deliberately narrow and vertical, with the dark
        // wood door directly below it.
        for x in -5..=-4 {
            for y in 2..=5 {
                scene.add_cube(x as f32, y as f32, 0.16, PURPLE_GLASS);
            }
        }
        for x in -5..=-4 {
            scene.add_cube(x as f32, 0., 0.16, DOOR);
            scene.add_cube(x as f32, 1., 0.16, DOOR);
        }

        // Central frame, entrance lintel and raised roof cap.
        for &x in &[-6, -3] {
            for y in 0..7 {
                scene.add_cube(x as f32, y as f32, 0.16, CONCRETE);
            }
        }
        for x in -6..=-3 {
            scene.add_cube(x as f32, 1., 0.18, TERRACOTTA);
            scene.add_cube(x as f32, 6., 0.16, CONCRETE);
        }
        for x in -6..=-3 {
            for z in -9..=0 {
                scene.add_cube(x as f32, 7., z as f32, CONCRETE);
            }
        }
    }
}
