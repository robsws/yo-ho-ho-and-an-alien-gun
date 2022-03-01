use bevy::{
    prelude::*, core::FixedTimestep, 
};

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
        .add_startup_system(player_setup)
        .add_startup_system(lighting_setup)
        .add_startup_system(debug_setup)
        .add_startup_system(scenery_setup)
        .add_system(
            player_input_handler
                .with_run_criteria(FixedTimestep::step(0.05))
                .label(PlayerMovement::Input)
                .before(PlayerMovement::Movement)
        )
        .add_system_set(
            SystemSet::new()
                .with_system(
                    ship_movement
                        .label(PlayerMovement::Movement)
                )
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

fn player_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    // Create a player location object
    commands.spawn_bundle(PbrBundle {
        ..Default::default()
    }).with_children(|location| {
        // Create the player ship
        location.spawn_bundle(PbrBundle {
            ..Default::default()
        }).with_children(|ship| {
            ship.spawn_bundle(PbrBundle {
                transform: Transform::from_translation(Vec3::new(0.75, 0.0, 0.0)),
                ..Default::default()
            }).with_children(|parent| {
                // player model
                parent.spawn_scene(
                    asset_server.load("models/pirate/ship_light.gltf#Scene0")
                );
            });
        })
        .insert(Ship {
            steering_wheel: SteeringWheel {
                angle: 0.0
            },
            speed: 0.1
        })
        .insert(Player {});
        // Create the camera, parented to player location
        location.spawn_bundle(make_camera());
    });
}

fn make_camera() -> OrthographicCameraBundle {
    let mut camera = OrthographicCameraBundle::new_3d();
    camera.orthographic_projection.scale = 15.0;
    camera.transform = Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y);
    camera
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

fn scenery_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    commands.spawn_bundle(PbrBundle {
        ..Default::default()
    }).with_children(|world| {
        world.spawn_bundle(PbrBundle {
            transform: Transform::from_translation(Vec3::new(5.0, 0.0, 5.0)),
            ..Default::default()
        }).with_children(|parent| {
            parent.spawn_scene(asset_server.load("models/nature/cliff_rock.glb#Scene0"));
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
    mut player_ships: Query<&mut Ship, With<Player>>,
    mut debug_text: Query<&mut Text, With<DebugText>>
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
            
            update_debug_text(debug_text, format!("{}", delta_angle));
            player_ship.steering_wheel.turn(delta_angle);
            prev_input.angle = new_angle;
        }
    }
}

fn ship_movement(
    mut ships: Query<(&Ship, &mut Transform)>,
    mut world_transforms: Query<&mut Transform, (With<World>, Without<Ship>)>
) {
    for (ship, mut ship_transform) in ships.iter_mut() {
        let rotation_angle = ship.steering_wheel.angle / 900.0;
        ship_transform.rotate(Quat::from_rotation_y(rotation_angle));
        let backward = ship_transform.local_z();
        if let Some(mut world_transform) = world_transforms.iter_mut().next() {
            world_transform.translation += backward * ship.speed;
        }
    }
}

fn update_debug_text(
    mut text_query: Query<&mut Text, With<DebugText>>,
    message: String
) {
    if let Some(mut text_box) = text_query.iter_mut().next() {
        text_box.sections[0].value = message;
    }
}