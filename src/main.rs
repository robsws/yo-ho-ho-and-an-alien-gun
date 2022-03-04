use bevy::{
    prelude::*, core::FixedTimestep, 
};
use bevy_prototype_debug_lines::*;
use bevy_rapier3d::{prelude::*, na::Vector3};

const ENEMY_CANNON_RANGE: f32 = 20.0;
const CANNONBALL_SPEED: f32 = 0.4;
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
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierRenderPlugin)
        .add_startup_system(camera_setup)
        .add_startup_system(player_setup)
        .add_startup_system(lighting_setup)
        .add_startup_system(debug_setup)
        .add_startup_system(enemy_setup)
        // Player input system
        .add_system(
            player_input_handler
                .with_run_criteria(FixedTimestep::step(0.05))
                .label(Pipeline::Input)
                .before(Pipeline::ShipMovement)
        )
        // // Enemy AI system
        .add_system(
            enemy_movement_ai
                .with_run_criteria(FixedTimestep::step(0.05))
                .label(Pipeline::AI)
                .before(Pipeline::ShipMovement)
        )
        .add_system(
            cannon_ai
                .with_run_criteria(FixedTimestep::step(0.05))
                .label(Pipeline::AI)
                .before(Pipeline::CannonballMovement)
        )
        // // Player movement system
        .add_system(
            ship_movement
                .label(Pipeline::ShipMovement)
        )
        .add_system(
            cannonball_tracking
                .label(Pipeline::CannonballMovement)
        )
        .run();
}

#[derive(Component)]
struct DebugText;

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
    .insert(DebugText);
}

#[derive(SystemLabel, Debug, Hash, PartialEq, Eq, Clone)]
enum Pipeline {
    Input,
    AI,
    ShipMovement,
    CannonballMovement
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Ship {
    steering_wheel: SteeringWheel,
    health: u32
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
    commands.spawn_bundle(RigidBodyBundle {
        position: Vec3::new(0.0, 0.0, 0.0).into(),
        forces: RigidBodyForces {
            gravity_scale: 0.0,
            // torque: Vec3::new(140.0, 80.0, 20.0).into(),
            ..Default::default()
        }.into(),
        damping: RigidBodyDamping { linear_damping: 3.0, angular_damping: 3.0 }.into(),
        mass_properties: (
            RigidBodyMassPropsFlags::TRANSLATION_LOCKED_Y |
            RigidBodyMassPropsFlags::ROTATION_LOCKED_X |
            RigidBodyMassPropsFlags::ROTATION_LOCKED_Z
        ).into(),
        ..Default::default()
    })
    .insert_bundle(ColliderBundle {
        shape: ColliderShape::cuboid(1.8, 2.0, 4.0).into(),
        collider_type: ColliderType::Solid.into(),
        material: ColliderMaterial { friction: 2.0, restitution: 0.1, ..Default::default() }.into(),
        mass_properties: ColliderMassProps::Density(4.0).into(),
        flags: ActiveEvents::CONTACT_EVENTS.into(),
        ..Default::default()
    })
    .insert(Transform::default())
    .insert(RigidBodyPositionSync::Discrete)
    .insert(RigidBodyTypeComponent::from(RigidBodyType::Dynamic))
    // .insert(ColliderDebugRender::with_id(1))
    .with_children(|ship| {
        // Add ship model
        ship.spawn_scene(
            asset_server.load("models/pirate/ship_light.glb#Scene0")
        );
    })
    .insert(Ship {
        steering_wheel: SteeringWheel {
            angle: 0.0,
        },
        health: 100
    })
    .insert(Player);
}


fn enemy_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    // Create enemies
    commands.spawn_bundle(RigidBodyBundle {
        position: Vec3::new(-20.0, 0.0, -15.0).into(),
        forces: RigidBodyForces {
            gravity_scale: 0.0,
            // torque: Vec3::new(140.0, 80.0, 20.0).into(),
            ..Default::default()
        }.into(),
        damping: RigidBodyDamping { linear_damping: 3.0, angular_damping: 3.0 }.into(),
        mass_properties: (
            RigidBodyMassPropsFlags::TRANSLATION_LOCKED_Y |
            RigidBodyMassPropsFlags::ROTATION_LOCKED_X |
            RigidBodyMassPropsFlags::ROTATION_LOCKED_Z
        ).into(),
        ..Default::default()
    })
    .insert_bundle(ColliderBundle {
        shape: ColliderShape::cuboid(1.8, 2.0, 4.0).into(),
        collider_type: ColliderType::Solid.into(),
        material: ColliderMaterial { friction: 2.0, restitution: 0.9, ..Default::default() }.into(),
        mass_properties: ColliderMassProps::Density(4.0).into(),
        ..Default::default()
    })
    .insert(Transform::default())
    .insert(RigidBodyPositionSync::Discrete)
    .insert(RigidBodyTypeComponent::from(RigidBodyType::Dynamic))
    // .insert(ColliderDebugRender::with_id(1))
    .with_children(|ship| {
        // Add ship model
        ship.spawn_scene(
            asset_server.load("models/pirate/ship_dark.glb#Scene0")
        );
    })
    .insert(Ship {
        steering_wheel: SteeringWheel {
            angle: 0.0
        },
        health: 20
    }).insert(Cannon {
        last_fired: 0.0
    });
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
    mut debug_text: Query<&mut Text, With<DebugText>>,

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
        update_debug_text(&mut debug_text, format!("health: {}", player_ship.health))
    }
}

fn ship_movement(
    mut ships: Query<(
        &Ship,
        &Transform,
        &mut RigidBodyForcesComponent,
        &mut RigidBodyMassPropsComponent
    )>,
    mut _debug_text: Query<&mut Text, With<DebugText>>
) {
    for (ship, t, mut rbf, mut rbmp) in ships.iter_mut() {
        let centre_of_rotation = t.translation + t.left() * (ship.steering_wheel.angle / 4.0);
        let lever_arm_vector = t.translation - centre_of_rotation;
        let torque = lever_arm_vector.cross(t.forward()) * 1000.0;
        rbmp.local_mprops.local_com = Vec3::new(0.0, 0.0, 1.0).into();
        rbf.force = (t.forward()*3000.0).into();
        rbf.torque = torque.into();
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

fn enemy_movement_ai(
    mut commands: Commands,
    mut enemy_ships: Query<(&mut Ship, &Transform), Without<Player>>,
    mut player_ts: Query<&Transform, With<Player>>,
    mut debug_text: Query<&mut Text, With<DebugText>>,
    mut lines: ResMut<DebugLines>
) {
    // Try and move into range of the player
    if let Some(player_t) = player_ts.iter().next() {
        for (mut enemy_ship, t) in enemy_ships.iter_mut() {
            let vec_to_player = player_t.translation - t.translation;
            let angle_to_player =
                t.forward().angle_between(vec_to_player);
            if is_to_left_of_player(player_t, t) {
                enemy_ship.steering_wheel.angle = angle_to_player * 6.0;
            } else {
                enemy_ship.steering_wheel.angle = angle_to_player * -6.0;
            }
            // lines.line(
            //     t.translation,
            //     t.translation + t.forward() * 10.0,
            //     0.0
            // );
        }
    }
}

fn cannon_ai(
    mut commands: Commands,
    mut player_ts: Query<&Transform, With<Player>>,
    mut cannons: Query<(&mut Cannon, &Transform), Without<Player>>,
    asset_server: Res<AssetServer>,
    mut debug_text: Query<&mut Text, With<DebugText>>,
    time: Res<Time>
) {
    if let Some(player_t) = player_ts.iter().next() {
        let now = time.seconds_since_startup();
        for (mut cannon, t) in cannons.iter_mut() {
            let to_player = player_t.translation - t.translation;
            let angle = t.forward().angle_between(to_player);
            if
                // cannon is off cooldown
                now - cannon.last_fired > CANNON_COOLDOWN // &&
                // // enemy is in range
                // t.translation.length() <= ENEMY_CANNON_RANGE &&
                // // player is either directly to left or right of enemy
                // angle > std::f32::consts::FRAC_PI_2 - 0.3 &&
                // angle < std::f32::consts::FRAC_PI_2 + 0.3
            {
                if is_to_left_of_player(player_t, t) {
                    // fire to the left
                    fire_cannon(&mut commands, t, t.left(), &asset_server, &mut debug_text);
                } else {
                    // fire to the right
                    fire_cannon(&mut commands, t, t.right(), &asset_server, &mut debug_text);
                }
                cannon.last_fired = time.seconds_since_startup();
            }
        }
    }
}

fn fire_cannon(
    commands: &mut Commands,
    enemy_transform: &Transform,
    direction: Vec3,
    asset_server: &Res<AssetServer>,
    debug_text: &mut Query<&mut Text, With<DebugText>>,
) {
    let cannonball = asset_server.load("models/pirate/cannonball.glb#Scene0");
    // commands.spawn_bundle(PbrBundle {
    //     transform: t,
    //     ..Default::default()
    commands.spawn_bundle(RigidBodyBundle {
        position: (enemy_transform.translation + direction * 2.0 + enemy_transform.up() * 2.0).into(),
        velocity: RigidBodyVelocity { 
            linvel: (direction * 5.0).into(),
            ..Default::default()
        }.into(),
        forces: RigidBodyForces {
            gravity_scale: 0.1,
            ..Default::default()
        }.into(),
        ..Default::default()
    })
    .insert_bundle(ColliderBundle {
        shape: ColliderShape::ball(0.5).into(),
        collider_type: ColliderType::Solid.into(),
        material: ColliderMaterial { friction: 0.7, restitution: 0.1, ..Default::default() }.into(),
        mass_properties: ColliderMassProps::Density(100.0).into(),
        ..Default::default()
    })
    .with_children(|parent| {
        parent.spawn_scene(cannonball);
    })
    .insert(Transform::default())
    .insert(RigidBodyPositionSync::Discrete)
    .insert(RigidBodyTypeComponent::from(RigidBodyType::Dynamic))
    .insert(Cannonball);
}

fn cannonball_tracking(
    mut commands: Commands,
    mut cannonballs: Query<(Entity, &Transform), With<Cannonball>>,
    mut ships: Query<(Entity, &mut Ship), With<Player>>,
    mut contact_events: EventReader<ContactEvent>,
    mut debug_text: Query<&mut Text, With<DebugText>>,
) {
    for (entity, t) in cannonballs.iter() {
        // cannonball drops into the sea
        if t.translation.y < 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
    for contact_event in contact_events.iter() {
        match contact_event {
            ContactEvent::Started(h1, h2) => {
                update_debug_text(&mut debug_text, "HIT!".to_string());
                if let Ok((cb_entity, _cb_t)) = cannonballs.get_mut(h1.entity()) {
                    commands.entity(cb_entity).despawn_recursive();
                }
                else if let Ok((_ship_ent, mut ship)) = ships.get_mut(h1.entity()) {
                    ship.health -= 10;
                }
                if let Ok((cb_entity, _cb_t)) = cannonballs.get_mut(h2.entity()) {
                    commands.entity(cb_entity).despawn_recursive();
                }
                else if let Ok((_ship_ent, mut ship)) = ships.get_mut(h2.entity()) {
                    ship.health -= 10;
                }
            },
            _ => ()
        };
    }
}

fn is_to_left_of_player(
    player_t: &Transform,
    other_t: &Transform
) -> bool {
    let right = other_t.right();
    let to_player = player_t.translation - other_t.translation;
    to_player.dot(right) < 0.0
}

// TODO
// - player laser
// - enemies spawning in
// - boundaries on map
// - score
// - sound effects and music
// - hud and game over screen
// - wheel animation

// -- submit --

// - simple visual effects
// - splash screen