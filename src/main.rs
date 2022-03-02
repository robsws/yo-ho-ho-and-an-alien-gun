use bevy::{
    prelude::*, core::FixedTimestep, 
};
use bevy_prototype_debug_lines::*;

const ENEMY_CANNON_RANGE: f32 = 20.0;
const CANNONBALL_SPEED: f32 = 0.2;
const CANNON_COOLDOWN: f64 = 3.0;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor {
            title: "Yo ho ho and an extra-terrestrial gun!".to_string(),
            width: 1000.0,
            height: 800.0,
            ..Default::default()
        })
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        })
        .insert_resource(ClearColor(Color::rgb(0.0, 0.4, 0.6)))
        .insert_resource(PreviousInput::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(DebugLinesPlugin::default())
        .add_startup_system(camera_setup)
        .add_startup_system(player_setup)
        .add_startup_system(lighting_setup)
        .add_startup_system(debug_setup)
        .add_startup_system(world_setup)
        // Player input system
        .add_system(
            player_input_handler
                .with_run_criteria(FixedTimestep::step(0.05))
                .label(PlayerMovement::Input)
                .before(PlayerMovement::Movement)
        )
        // Enemy AI system
        .add_system(
            enemy_movement_ai
                .with_run_criteria(FixedTimestep::step(0.05))
                .label(EnemyMovement::AI)
                .before(EnemyMovement::Movement)
        )
        .add_system(
            cannon_ai
                .with_run_criteria(FixedTimestep::step(0.05))
                .label(CannonballMovement::AI)
                .before(CannonballMovement::Movement)
        )
        // Player movement system
        .add_system(
            player_movement
                .label(PlayerMovement::Movement)
        )
        // .add_system(
        //     enemy_movement
        //         .label(EnemyMovement::Movement)
        // )
        .add_system(
            cannonball_movement
                .label(CannonballMovement::Movement)
        )
        .run();
}

#[derive(Component)]
struct DebugText {
    message: String
}

fn debug_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    let font = asset_server.load("fonts/Arial Unicode.ttf");
    commands.spawn_bundle(UiCameraBundle::default());
    commands.spawn_bundle(TextBundle {
        style: Style {
            align_self: AlignSelf::FlexEnd,
            position_type: PositionType::Absolute,
            position: Rect {
                top: Val::Px(5.0),
                left: Val::Px(15.0),
                ..Default::default()
            },
            ..Default::default()
        },
        text: Text::with_section(
            "Some debug text",
            TextStyle {
                font: font.clone(),
                font_size: 50.0,
                color: Color::WHITE,
            },
            Default::default(),
        ),
        ..Default::default()
    })
    .insert(DebugText { message: "debug".to_string() });
}

#[derive(SystemLabel, Debug, Hash, PartialEq, Eq, Clone)]
enum PlayerMovement {
    Input,
    Movement
}

#[derive(SystemLabel, Debug, Hash, PartialEq, Eq, Clone)]
enum EnemyMovement{
    AI,
    Movement
}

#[derive(SystemLabel, Debug, Hash, PartialEq, Eq, Clone)]
enum CannonballMovement{
    AI,
    Movement
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Ship {
    steering_wheel: SteeringWheel,
    speed: f32,
}

#[derive(Component)]
struct Cannon {
    last_fired: f64
}

struct SteeringWheel {
    angle: f32
}

impl SteeringWheel {
    fn turn(&mut self, delta_angle: f32) {
        self.angle += delta_angle;
        self.angle = self.angle.clamp(
            -std::f32::consts::TAU * 3.0, 
            std::f32::consts::TAU * 3.0
        );
    }
}

#[derive(Component)]
struct Cannonball;

fn player_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    // Create the player ship
    commands.spawn_bundle(PbrBundle {
        ..Default::default()
    }).with_children(|ship| {
        ship.spawn_scene(
            asset_server.load("models/pirate/ship_light.glb#Scene0")
        );
    })
    .insert(Ship {
        steering_wheel: SteeringWheel {
            angle: 0.0
        },
        speed: 0.1
    })
    .insert(Player {});
}

fn camera_setup(
    mut commands: Commands
) {
    let mut camera = OrthographicCameraBundle::new_3d();
    camera.orthographic_projection.scale = 15.0;
    camera.transform = Transform::from_xyz(60.0, 60.0, 60.0).looking_at(Vec3::ZERO, Vec3::Y);
    commands.spawn_bundle(camera);
}

fn lighting_setup(
    mut commands: Commands
) {
    // commands.spawn_bundle(PointLightBundle {
    //     transform: Transform::from_xyz(4.0, 5.0, 4.0),
    //     ..Default::default()
    // });
    const HALF_SIZE: f32 = 1.0;
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadow_projection: OrthographicProjection {
                left: -HALF_SIZE,
                right: HALF_SIZE,
                bottom: -HALF_SIZE,
                top: HALF_SIZE,
                near: -10.0 * HALF_SIZE,
                far: 10.0 * HALF_SIZE,
                ..Default::default()
            },
            shadows_enabled: true,
            ..Default::default()
        },
        ..Default::default()
    });
}

#[derive(Component)]
struct World;

fn world_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    // Load the cannonball
    let _cannonball: Handle<Scene> =
        asset_server.load("models/pirate/cannonball.glb#Scene0");
    // Load the world
    commands.spawn_bundle(PbrBundle {
        ..Default::default()
    }).with_children(|world| {
        // Make the scenery
        world.spawn_bundle(PbrBundle {
            transform: Transform::from_translation(Vec3::new(5.0, 0.0, 5.0)),
            ..Default::default()
        }).with_children(|parent| {
            parent.spawn_scene(asset_server.load("models/nature/cliff_rock.glb#Scene0"));
        });
        world.spawn_bundle(PbrBundle {
            transform: Transform::from_translation(Vec3::new(-5.0, 0.0, 0.0)),
            ..Default::default()
        }).with_children(|parent| {
            parent.spawn_scene(asset_server.load("models/nature/cliff_rock.glb#Scene0"));
        });
        // Create enemies
        world.spawn_bundle(PbrBundle {
            transform: Transform::from_translation(Vec3::new(-10.0, 0.0, 0.0)),
            ..Default::default()
        }).with_children(|parent| {
            parent.spawn_scene(
                asset_server.load("models/pirate/ship_dark.glb#Scene0")
            );
        }).insert(Ship {
            steering_wheel: SteeringWheel { angle: 0.0 },
            speed: 0.0
        }).insert(Cannon {
            last_fired: 0.0
        });
    }).insert(World);
}

#[derive(Default)]
struct PreviousInput {
    angle: f32
}

fn player_input_handler(
    gamepads: Res<Gamepads>,
    // button_inputs: Res<Input<GamepadButton>>,
    // button_axes: Res<Axis<GamepadButton>>,
    axes: Res<Axis<GamepadAxis>>,
    mut prev_input: ResMut<PreviousInput>,
    mut player_ships: Query<&mut Ship, With<Player>>
) {
    if let Some(mut player_ship) = player_ships.iter_mut().next() {
        if let Some(gamepad) = gamepads.iter().next() {
            let mut new_angle = prev_input.angle;
            let left_stick_x = axes
                .get(GamepadAxis(*gamepad, GamepadAxisType::LeftStickX))
                .unwrap();
            let left_stick_y = axes
                .get(GamepadAxis(*gamepad, GamepadAxisType::LeftStickY))
                .unwrap();
            if left_stick_x.abs() > 0.5 || left_stick_y.abs() > 0.5 {
                new_angle = left_stick_y.atan2(left_stick_x);
            }
            let delta_angle = new_angle - prev_input.angle;
            // Handle the cases where the delta crosses the PI boundary at 180 degrees
            let delta_angle = 
                if delta_angle > std::f32::consts::PI {
                    delta_angle - std::f32::consts::TAU
                } else if delta_angle < -std::f32::consts::PI {
                    delta_angle + std::f32::consts::TAU
                } else {
                    delta_angle
                };
            
            player_ship.steering_wheel.turn(delta_angle);
            prev_input.angle = new_angle;
        }
    }
}

fn player_movement(
    mut player_ships: Query<(&Ship, &mut Transform), With<Player>>,
    mut world_transforms: Query<&mut Transform, (With<World>, Without<Ship>)>,
    mut _debug_text: Query<&mut Text, With<DebugText>>,
) {
    for (ship, mut t) in player_ships.iter_mut() {
        let rotation_angle = ship.steering_wheel.angle / 900.0;
        t.rotate(Quat::from_rotation_y(rotation_angle));
        let backward = t.local_z();
        if let Some(mut wt) = world_transforms.iter_mut().next() {
            wt.translation += backward * ship.speed;
        }
    }
}

fn enemy_movement(
    mut enemy_ships: Query<(&Ship, &mut Transform), Without<Player>>,
    mut _debug_text: Query<&mut Text, With<DebugText>>
) {
    for (ship, mut t) in enemy_ships.iter_mut() {
        let rotation_angle = ship.steering_wheel.angle / 900.0;
        t.rotate(Quat::from_rotation_y(rotation_angle));
        let forward = t.forward();
        // update_debug_text(&mut debug_text, format!("{:?}", forward));
        t.translation += forward * ship.speed;
    }
}

fn update_debug_text(
    text_query: &mut Query<&mut Text, With<DebugText>>,
    message: String
) {
    if let Some(mut text_box) = text_query.iter_mut().next() {
        text_box.sections[0].value = message;
    }
}

fn enemy_debug(
    mut enemy_ships: Query<(&mut Ship, &GlobalTransform), Without<Player>>,
    time: Res<Time>,
) {
    for (mut enemy_ship, enemy_transform) in enemy_ships.iter_mut() {
        enemy_ship.steering_wheel.angle = time.seconds_since_startup().sin() as f32 * 18.0;
    }
}

fn enemy_movement_ai(
    mut commands: Commands,
    mut enemy_ships: Query<(&mut Ship, &GlobalTransform), Without<Player>>,
    mut debug_text: Query<&mut Text, With<DebugText>>,
    mut lines: ResMut<DebugLines>
) {
    // Try and move into range of the player
    let commands_ref = &mut commands;
    for (mut enemy_ship, gt) in enemy_ships.iter_mut() {
        let angle_to_player =
            gt.forward().angle_between(-gt.translation);
        if is_to_left_of_player(gt) {
            enemy_ship.steering_wheel.angle = angle_to_player * 6.0;
        } else {
            enemy_ship.steering_wheel.angle = angle_to_player * -6.0;
        }
        lines.line(
            gt.translation, 
            gt.translation + gt.forward() * 10.0,
            0.0
        );
    }
}

fn cannon_ai(
    mut commands: Commands,
    worlds: Query<Entity, With<World>>,
    mut cannons: Query<(&mut Cannon, &GlobalTransform, &Transform), Without<Player>>,
    asset_server: Res<AssetServer>,
    mut debug_text: Query<&mut Text, With<DebugText>>,
    time: Res<Time>
) {
    if let Some(world) = worlds.iter().next() {
        let now = time.seconds_since_startup();
        for (mut cannon, gt, t) in cannons.iter_mut() {
            let angle = gt.forward().angle_between(-gt.translation);
            if
                // cannon is off cooldown
                now - cannon.last_fired > CANNON_COOLDOWN &&
                // enemy is in range
                gt.translation.length() <= ENEMY_CANNON_RANGE &&
                // player is either directly to left or right of enemy
                angle > std::f32::consts::FRAC_PI_2 - 0.1 &&
                angle < std::f32::consts::FRAC_PI_2 + 0.1
            {
                if is_to_left_of_player(gt) {
                    // fire to the left
                    fire_cannon(&mut commands, world, t, t.left(), &asset_server, &mut debug_text);
                } else {
                    // fire to the right
                    fire_cannon(&mut commands, world, t, t.right(), &asset_server, &mut debug_text);
                }
                cannon.last_fired = time.seconds_since_startup();
            }
        }
    }
}

fn fire_cannon(
    commands: &mut Commands,
    world: Entity,
    enemy_transform: &Transform,
    direction: Vec3,
    asset_server: &Res<AssetServer>,
    debug_text: &mut Query<&mut Text, With<DebugText>>,
) {
    let cannonball = asset_server.load("models/pirate/cannonball.glb#Scene0");
    commands.entity(world).with_children(|parent| {
        let mut cannonball_transform = enemy_transform.clone();
        cannonball_transform.look_at(
            cannonball_transform.translation + direction,
            cannonball_transform.up()
        );
        cannonball_transform.apply_non_uniform_scale(Vec3::splat(2.0));
        parent.spawn_bundle(PbrBundle {
            transform: cannonball_transform,
            ..Default::default()
        }).with_children(|parent| {
            parent.spawn_scene(cannonball);
        })
        .insert(Cannonball);
    });
}

fn cannonball_movement(
    cannonball_entities: Query<Entity, With<Cannonball>>,
    mut cannonball_transforms: Query<(&mut Transform, &Cannonball)>
) {
    let mut i = 0;
    for entity in cannonball_entities.iter() {
        let (mut transform, cannonball) = cannonball_transforms.get_mut(entity).unwrap();
        let forward = transform.forward();
        transform.translation += forward * CANNONBALL_SPEED;
        i += 1;
    }
    info!("{} cannonballs", i);
}

fn is_to_left_of_player(
    enemy_transform: &GlobalTransform
) -> bool {
    let right = enemy_transform.right();
    (-enemy_transform.translation).dot(right) < 0.0
}