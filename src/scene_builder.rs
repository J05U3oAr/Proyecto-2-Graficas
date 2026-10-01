use crate::materials::default_materials;
use crate::scene::Scene;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum House {
    Ve7,
    Elru,
}

impl House {
    pub const ALL: [Self; 2] = [Self::Ve7, Self::Elru];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Ve7 => "Ve7",
            Self::Elru => "elru",
        }
    }
}

pub struct SceneBuilder;

impl SceneBuilder {
    /// A cubic miniature world: every primitive is a unit cube.
    pub fn build_planet_preview(house: House) -> Scene {
        let mut scene = Scene::new(default_materials());
        for x in -8i32..=8 {
            for z in -8i32..=8 {
                for y in -12..=4 {
                    // A solid cubic world with exposed strata on all four sides.
                    let material = if y == 4 {
                        if x >= 5 && z >= -3 {
                            14
                        } else if x == -1 || x == 0 {
                            11
                        } else {
                            0
                        }
                    } else if y >= 1 {
                        1
                    } else if (x * 7 + z * 11 + y * 3).rem_euclid(23) == 0 {
                        7
                    } else {
                        2
                    };
                    scene.add_cube(x as f32, y as f32, z as f32, material);
                }
            }
        }

        match house {
            House::Ve7 => {
                // The two cream wings, purple windows and taller entrance echo Ve7.
                for x in -6..=4 {
                    for z in -6..=-2 {
                        let central = (-2..=0).contains(&x);
                        let roof = if central { 11 } else { 9 };
                        for y in 5..=roof {
                            let front = z == -2;
                            let material = if front && central && x == -1 && y < 7 {
                                3
                            } else if front && y > 5 && y < roof && y != 7 {
                                9
                            } else if y == 7 && !central {
                                8
                            } else {
                                7
                            };
                            scene.add_cube(x as f32, y as f32, z as f32, material);
                        }
                    }
                }
                for &(x, z) in &[(-6, 4), (3, 5), (-6, -7)] {
                    for y in 5..8 {
                        scene.add_cube(x as f32, y as f32, z as f32, 10);
                    }
                    for dx in -1..=1 {
                        for dz in -1..=1 {
                            for y in 8..=9 {
                                scene.add_cube((x + dx) as f32, y as f32, (z + dz) as f32, 4);
                            }
                        }
                    }
                }
                // Cubic stepping stones over the pond.
                for z in 1..=3 {
                    scene.add_cube(6., 5., z as f32, 3);
                }
            }
            House::Elru => Self::build_elru_preview_house(&mut scene),
        }
        scene.rebuild_bvh();
        scene
    }

    pub fn build(house: House) -> Scene {
        if house == House::Elru {
            return Self::build_elru_world();
        }
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

        match house {
            House::Ve7 => {
                Self::build_vegeta_house(&mut scene);
                Self::build_right_balcony(&mut scene);
            }
            House::Elru => unreachable!("Elru returns its dedicated scene above"),
        }
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

    fn build_elru_preview_house(scene: &mut Scene) {
        const WHITE: usize = 7;
        const GLASS: usize = 6;
        const WATER: usize = 5;
        const WOOD: usize = 3;

        // Small square lake in front of the house.
        for x in 1..=7 {
            for z in 0..=5 {
                if (x - 4) * (x - 4) + (z - 2) * (z - 2) <= 10 {
                    scene.add_cube(x as f32, 5., z as f32, WATER);
                }
            }
        }

        for &(left, right, roof) in &[(-6, -2, 10), (2, 6, 11)] {
            for x in left..=right {
                for z in -6..=-2 {
                    for y in 5..=roof {
                        let front_window = z == -2 && x != left && x != right && y > 6 && y < roof;
                        scene.add_cube(
                            x as f32,
                            y as f32,
                            z as f32,
                            if front_window { GLASS } else { WHITE },
                        );
                    }
                }
            }
        }
        for x in -1..=1 {
            for z in -6..=-2 {
                for y in 5..=13 {
                    scene.add_cube(
                        x as f32,
                        y as f32,
                        z as f32,
                        if z == -2 && y > 6 { GLASS } else { WHITE },
                    );
                }
            }
        }
        for z in -1..=2 {
            scene.add_cube(0., 5., z as f32, WOOD);
        }
    }

    fn build_elru_world() -> Scene {
        const GRASS: usize = 0;
        const DIRT: usize = 1;
        const STONE: usize = 2;
        const WOOD: usize = 3;
        const WATER: usize = 5;
        const GLASS: usize = 6;
        const WHITE: usize = 7;
        const ACCENT: usize = 8;
        const RADIUS: i32 = 26;

        let mut scene = Scene::new(default_materials());

        // The island's round outline is approximated exclusively by a stepped
        // field of cubes. Its exposed stone and dirt layers make it read as a
        // floating diorama from every angle.
        for x in -RADIUS..=RADIUS {
            for z in -RADIUS..=RADIUS {
                let distance_squared = x * x + z * z;
                if distance_squared > RADIUS * RADIUS {
                    continue;
                }
                for y in -4..=-1 {
                    scene.add_cube(
                        x as f32,
                        y as f32,
                        z as f32,
                        if y == -4 { STONE } else { DIRT },
                    );
                }
                let border = distance_squared >= 23 * 23;
                scene.add_cube(x as f32, -1., z as f32, if border { WHITE } else { GRASS });
            }
        }

        // A compact, blocky lake faces the house; a one-cube white edge frames it.
        for x in -10i32..=10 {
            for z in 6i32..=22 {
                let distance_squared = x * x + (z - 14) * (z - 14);
                if distance_squared <= 64 {
                    scene.add_cube(x as f32, -0.92, z as f32, WATER);
                } else if distance_squared <= 82 {
                    scene.add_cube(x as f32, -0.90, z as f32, WHITE);
                }
            }
        }

        // Lower white terraces connect the water, lawn and entrance.
        for x in -15..=15 {
            for z in -2..=3 {
                scene.add_cube(x as f32, 0., z as f32, WHITE);
            }
        }
        for step in 0..6 {
            for x in -2..=2 {
                scene.add_cube(x as f32, (step / 2) as f32, (4 + step) as f32, WOOD);
            }
        }

        Self::build_elru_modern_house(&mut scene, WHITE, GLASS, ACCENT);

        // Cubic shrubs at the corners balance the tall facade without adding
        // any non-cube geometry.
        for &(tree_x, tree_z) in &[(-18, -3), (18, -3), (-16, 17), (16, 17)] {
            for y in 0..4 {
                scene.add_cube(tree_x as f32, y as f32, tree_z as f32, WOOD);
            }
            for x in tree_x - 1..=tree_x + 1 {
                for z in tree_z - 1..=tree_z + 1 {
                    for y in 3..=5 {
                        scene.add_cube(x as f32, y as f32, z as f32, 4);
                    }
                }
            }
        }

        scene.rebuild_bvh();
        scene
    }

    fn build_elru_modern_house(scene: &mut Scene, white: usize, glass: usize, accent: usize) {
        // Two offset wings form the wide floating frames from the reference.
        for &(left, right, top) in &[(-14, -4, 9), (4, 14, 10)] {
            for x in left..=right {
                for z in -14..=-3 {
                    for y in 1..=top {
                        let front = z == -3;
                        let window = front
                            && x > left + 1
                            && x < right - 1
                            && y > 3
                            && y < top - 1;
                        let balcony_shadow = front && y == 3 && x > left && x < right;
                        let material = if window {
                            glass
                        } else if balcony_shadow {
                            accent
                        } else {
                            white
                        };
                        scene.add_cube(x as f32, y as f32, z as f32, material);
                    }
                }
            }
            // Raised outer frame projects beyond the glazed face. The left
            // wing keeps a clean, uninterrupted corner so the new glass return
            // can read as one continuous architectural element.
            for y in 1..=top + 1 {
                scene.add_cube(left as f32, y as f32, -2., white);
                scene.add_cube(right as f32, y as f32, -2., white);
            }
            for x in left..=right {
                scene.add_cube(x as f32, 1., -2., white);
                scene.add_cube(x as f32, (top + 1) as f32, -2., white);
            }
        }

        // The taller transparent central spine joins both floating volumes.
        for x in -3..=3 {
            for z in -13..=-3 {
                for y in 1..=14 {
                    let front_glass = z == -3 && x != -3 && x != 3 && (3..=12).contains(&y);
                    scene.add_cube(
                        x as f32,
                        y as f32,
                        z as f32,
                        if front_glass { glass } else { white },
                    );
                }
            }
        }
        for y in 10..=14 {
            scene.add_cube(-4., y as f32, -2., white);
            scene.add_cube(4., y as f32, -2., white);
        }
        for x in -4..=4 {
            scene.add_cube(x as f32, 14., -2., white);
        }

        // Square balcony rails and recessed entry details.
        for &(from, to, y) in &[(6, 12, 6)] {
            for x in from..=to {
                scene.add_cube(x as f32, y as f32, -1., glass);
            }
        }
        Self::build_elru_left_corner_modernization(scene, white, glass, accent);
        for y in 1..=3 {
            for x in -2..=2 {
                scene.add_cube(x as f32, y as f32, -2., if y == 1 { accent } else { glass });
            }
        }
        Self::build_elru_rear_facade(scene, white, glass);
        Self::build_elru_right_facade(scene, white, glass);
    }

    fn build_elru_left_corner_modernization(
        scene: &mut Scene,
        white: usize,
        glass: usize,
        accent: usize,
    ) {
        // The left wing is the principal modern intervention. A thin glass
        // plane sits in front of the existing front opening and returns along
        // the outside wall until it meets the rear facade. This creates the
        // continuous front-to-back corner glazing requested in the redesign.
        scene.add_box(-13., 4., -2.08, 8., 4., 0.10, glass);
        scene.add_box(-14.08, 4., -15., 0.10, 4., 12., glass);

        // A shallow cantilever replaces the bulky cube balcony. The darker
        // underside gives the overhang a crisp shadow line while the clear
        // railing keeps the corner visually light.
        scene.add_box(-13., 4.05, -4.25, 8., 0.25, 2.45, accent);
        scene.add_box(-13., 4.30, -4.25, 8., 0.12, 2.45, white);
        scene.add_box(-13., 4.42, -1.92, 8., 0.08, 0.08, glass);
        scene.add_box(-13., 4.42, -4.17, 8., 0.08, 0.08, glass);
        scene.add_box(-13., 4.42, -4.17, 0.08, 1.20, 2.33, glass);
        scene.add_box(-5.08, 4.42, -4.17, 0.08, 1.20, 2.33, glass);
        scene.add_box(-13., 5.62, -1.92, 8., 0.08, 0.08, glass);
    }

    fn build_elru_right_facade(scene: &mut Scene, white: usize, glass: usize) {
        const RIGHT_WALL: f32 = 15.;

        // This is the right-hand elevation when the lake/front facade is in
        // view. The side is a tall screen of square white ribs and vertical
        // glass slots, matching the rear tower at the same corner.
        for z in -14..=-3 {
            for y in 1..=13 {
                let slot = [-13, -11, -9, -7, -5].contains(&z) && (2..=13).contains(&y);
                scene.add_cube(
                    RIGHT_WALL,
                    y as f32,
                    z as f32,
                    if slot { glass } else { white },
                );
            }
        }

        // Three cubical roof steps give the tall side the same staggered top
        // profile seen in the reference, without introducing any other shape.
        for z in -14..=-11 {
            for y in 14..=15 {
                scene.add_cube(RIGHT_WALL, y as f32, z as f32, white);
            }
        }
        for z in -10..=-7 {
            for y in 14..=14 {
                scene.add_cube(RIGHT_WALL, y as f32, z as f32, white);
            }
        }
    }

    fn build_elru_rear_facade(scene: &mut Scene, white: usize, glass: usize) {
        const BACK: f32 = -15.;
        const GLASS_FACE: f32 = BACK;

        // Left service tower: four white ribs separated by tall glass slots.
        for x in 7..=14 {
            for y in 1..=15 {
                let slot = [13, 11, 9].contains(&x) && (2..=13).contains(&y);
                scene.add_cube(x as f32, y as f32, BACK, if slot { glass } else { white });
            }
        }
        for x in 7..=14 {
            scene.add_cube(x as f32, 15., GLASS_FACE, white);
        }

        // A narrow, taller glazed spine links the left tower to the main house.
        for x in -3..=3 {
            for y in 1..=16 {
                let window = (-2..=2).contains(&x) && (2..=15).contains(&y);
                scene.add_cube(x as f32, y as f32, BACK, if window { glass } else { white });
            }
        }
        for x in -3..=3 {
            scene.add_cube(x as f32, 16., GLASS_FACE, white);
        }

        // Right rear volume: two large stacked glass openings framed in white.
        for x in -17..=-3 {
            for y in 1..=13 {
                let upper = (-13..=-6).contains(&x) && (9..=11).contains(&y);
                let lower = (-14..=-5).contains(&x) && (4..=7).contains(&y);
                let vertical = (-15..=-6).contains(&x) && x % 2 != 0 && (1..=3).contains(&y);
                scene.add_cube(
                    x as f32,
                    y as f32,
                    BACK,
                    if upper || lower || vertical {
                        glass
                    } else {
                        white
                    },
                );
            }
        }

        for x in -17..=-3 {
            scene.add_cube(x as f32, 13., GLASS_FACE, white);
        }

        // Deep square surrounds make the two rear windows read as recessed
        // openings instead of glass pasted on top of the wall.
        Self::frame_elru_window(scene, -14, -5, 8, 12, -16., white);
        Self::frame_elru_window(scene, -15, -4, 3, 8, -16., white);

        // The lower vertical glazing receives one continuous sill and two side
        // jambs, matching the tall window rhythm above it.
        for x in -16..=-5 {
            scene.add_cube(x as f32, 0., -16., white);
        }
        for y in 0..=3 {
            scene.add_cube(-16., y as f32, -16., white);
            scene.add_cube(-5., y as f32, -16., white);
        }
    }

    fn frame_elru_window(
        scene: &mut Scene,
        min_x: i32,
        max_x: i32,
        min_y: i32,
        max_y: i32,
        z: f32,
        material: usize,
    ) {
        for x in min_x..=max_x {
            scene.add_cube(x as f32, min_y as f32, z, material);
            scene.add_cube(x as f32, max_y as f32, z, material);
        }
        for y in min_y..=max_y {
            scene.add_cube(min_x as f32, y as f32, z, material);
            scene.add_cube(max_x as f32, y as f32, z, material);
        }
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
