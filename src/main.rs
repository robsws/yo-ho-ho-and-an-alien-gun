use bevy::{
    prelude::*, core::FixedTimestep, 
};
use bevy_prototype_debug_lines::*;

const ENEMY_CANNON_RANGE: f32 = 20.0;

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
            enemy_ai
                .with_run_criteria(FixedTimestep::step(0.05))
                .label(EnemyMovement::AI)
                .before(EnemyMovement::Movement)
        )
        // Player movement system
        .add_system(
            player_movement
                .label(PlayerMovement::Movement)
        )
        .add_system(
            enemy_movement
                .label(EnemyMovement::Movement)
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

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Ship {
    steering_wheel: SteeringWheel,
    speed: f32
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

struct CannonballModel {
    model: Handle<Scene>
}

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
        asset_server.load("models/pirate/cannonball.gltf#Scene0");
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
    for (ship, mut ship_transform) in player_ships.iter_mut() {
        info!("{}", ship.steering_wheel.angle);
        let rotation_angle = ship.steering_wheel.angle / 900.0;
        ship_transform.rotate(Quat::from_rotation_y(rotation_angle));
        let backward = ship_transform.local_z();
        if let Some(mut world_transform) = world_transforms.iter_mut().next() {
            world_transform.translation += backward * ship.speed;
        }
    }
}

fn enemy_movement(
    mut enemy_ships: Query<(&Ship, &mut Transform), Without<Player>>,
    mut _debug_text: Query<&mut Text, With<DebugText>>
) {
    for (ship, mut ship_transform) in enemy_ships.iter_mut() {
        let rotation_angle = ship.steering_wheel.angle / 900.0;
        ship_transform.rotate(Quat::from_rotation_y(rotation_angle));
        let forward = ship_transform.forward();
        // update_debug_text(&mut debug_text, format!("{:?}", forward));
        ship_transform.translation += forward * ship.speed;
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

fn enemy_ai(
    mut enemy_ships: Query<(&mut Ship, &GlobalTransform), Without<Player>>,
    mut debug_text: Query<&mut Text, With<DebugText>>,
    asset_server: Res<AssetServer>,
    mut lines: ResMut<DebugLines>
) {
    // Try and move into range of the player so that they
    // are side on, so that they can fire cannons
    // i.e. they want to be at a tangent to the circle surrounding the
    // player of radius cannon range
    for (mut enemy_ship, enemy_transform) in enemy_ships.iter_mut() {
        // align_enemy_to_player_vector(
        //     &mut enemy_ship,
        //     enemy_transform,
        //     &mut debug_text
        // );
        fire_cannon(
            &enemy_transform,
            &asset_server,
            &mut debug_text
        );
        lines.line(enemy_transform.translation, enemy_transform.translation + enemy_transform.forward() * 10.0, 0.0);
    }
}

fn align_enemy_to_player_vector(
    enemy: &mut Ship,
    transform: &GlobalTransform,
    debug_text: &mut Query<&mut Text, With<DebugText>>,
) {
    let angle_to_player = transform.forward().angle_between(-transform.translation);
    if is_to_left_of_player(transform) {
        enemy.steering_wheel.angle = angle_to_player * 6.0;
    } else {
        enemy.steering_wheel.angle = angle_to_player * -6.0;
    }
}

fn fire_cannon(
    enemy_transform: &GlobalTransform,
    asset_server: &Res<AssetServer>,
    debug_text: &mut Query<&mut Text, With<DebugText>>,
) {
    let angle = enemy_transform.forward().angle_between(-enemy_transform.translation);
    if enemy_transform.translation.length() <= ENEMY_CANNON_RANGE {
        if angle > std::f32::consts::FRAC_PI_2 - 0.1 && angle < std::f32::consts::FRAC_PI_2 + 0.1 {
            if is_to_left_of_player(enemy_transform) {
                update_debug_text(debug_text, format!("angle: {:?}\ndist: {:?}\n FIRE LEFT!",angle,enemy_transform.translation.length()));
            } else {
                update_debug_text(debug_text, format!("angle: {:?}\ndist: {:?}\n FIRE RIGHT!",angle,enemy_transform.translation.length()));
            }
        } else {
            update_debug_text(debug_text, format!("angle: {:?}\ndist: {:?}\n angle not aligned",angle,enemy_transform.translation.length()));
        }
    } else {
        update_debug_text(debug_text, format!("angle: {:?}\ndist: {:?}\n out of range",angle,enemy_transform.translation.length()));
    }
}

fn is_to_left_of_player(
    enemy_transform: &GlobalTransform
) -> bool {
    let right = enemy_transform.right();
    (-enemy_transform.translation).dot(right) < 0.0
}