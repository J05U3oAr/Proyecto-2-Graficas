use crate::materials::default_materials;
use crate::scene::Scene;

pub struct SceneBuilder;

impl SceneBuilder {
    pub fn build() -> Scene {
        let mut scene = Scene::new(default_materials());

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

        for &(tree_x, tree_z) in &[(-8, -6), (7, -5), (7, 5)] {
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

        for &(x, z) in &[(-5, 2), (-5, 5), (5, 2), (5, 6), (-2, 7)] {
            scene.add_cube(x as f32, 0., z as f32, 2);
        }
        scene.rebuild_bvh();
        scene
    }
}
