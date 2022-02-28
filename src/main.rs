use bevy::{
    prelude::*, 
    core::FixedTimestep
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
        .add_plugins(DefaultPlugins)
        .add_startup_system(player_setup)
        .add_startup_system(lighting_setup)
        .add_system(
            player_input_handler
                .label(PlayerMovement::Input)
                .before(PlayerMovement::Movement)
        )
        .add_system_set(
            SystemSet::new()
                .with_system(
                    player_movement
                        .label(PlayerMovement::Movement)
                )
        )
        .run();
}

#[derive(SystemLabel, Debug, Hash, PartialEq, Eq, Clone)]
enum PlayerMovement {
    Input,
    Movement
}

#[derive(Component)]
struct PlayerShip {
    direction_radians: f32
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
            // player model
            ship.spawn_scene(
                asset_server.load("models/pirate/ship_light.gltf#Scene0")
            );
        })
        .insert(PlayerShip { direction_radians: 0.0 });
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

fn player_input_handler(
    gamepads: Res<Gamepads>,
    button_inputs: Res<Input<GamepadButton>>,
    button_axes: Res<Axis<GamepadButton>>,
    axes: Res<Axis<GamepadAxis>>,
    mut players: Query<&mut PlayerShip>
) {
    if let Some(mut player) = players.iter_mut().next() {
        for gamepad in gamepads.iter().cloned() {
            let left_stick_x = axes
                .get(GamepadAxis(gamepad, GamepadAxisType::LeftStickX))
                .unwrap();
            let left_stick_y = axes
                .get(GamepadAxis(gamepad, GamepadAxisType::LeftStickY))
                .unwrap();
            if left_stick_x.abs() > 0.01 || left_stick_y.abs() > 0.01 {
                player.direction_radians = left_stick_y.atan2(left_stick_x);
            }
        }
    }
}

fn player_movement(
    mut players: Query<(&PlayerShip, &mut Transform)>
) {
    if let Some(mut player) = players.iter_mut().next() {
        player.1.rotation = Quat::from_rotation_y(
            player.0.direction_radians - std::f32::consts::FRAC_PI_4
        );
    }
}