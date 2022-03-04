use bevy::{
    prelude::*, core::FixedTimestep
};
use bevy_prototype_debug_lines::*;
use bevy_rapier3d::prelude::*;

use std::f32::consts;

const ENEMY_CANNON_RANGE: f32 = 20.0;
const CANNONBALL_SPEED: f32 = 0.4;
const CANNON_COOLDOWN: f64 = 5.0;
const LASER_COOLDOWN: f64 = 1.0;
const LASER_TIMEOUT: f64 = 0.3;
const SPAWN_COOLDOWN: f64 = 5.0;
const ENEMY_COUNT: i32 = 10;

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
        .insert_resource(EnemyCounter {to_spawn: ENEMY_COUNT, dead: 0})
        .add_plugins(DefaultPlugins)
        .add_plugin(DebugLinesPlugin::default())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierRenderPlugin)
        .add_startup_system(camera_setup)
        .add_startup_system(player_setup)
        .add_startup_system(lighting_setup)
        .add_startup_system(hud_setup)
        .add_startup_system(spawner_setup)
        // Player input system
        .add_system(
            player_input_handler
                .with_run_criteria(FixedTimestep::step(0.05))
                .label(Pipeline::Input)
                .before(Pipeline::ShipMovement)
        )
        .add_system(
            laser_gun_handler
                .label(Pipeline::Input)
        )
        .add_system(
            enemy_spawner
                .label(Pipeline::Spawner)
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
        .add_system(
            laser_cleanup
                .label(Pipeline::LaserCleanup)
                .after(Pipeline::Input)
        )
        .add_system(
            hud_handler
                .label(Pipeline::HUD)
                .after(Pipeline::ShipMovement)
                .after(Pipeline::Input)
                .after(Pipeline::CannonballMovement)
                .after(Pipeline::AI)
        )
        .run();
}

#[derive(Component)]
struct HUD;

fn hud_setup(
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
    .insert(HUD);
}

#[derive(SystemLabel, Debug, Hash, PartialEq, Eq, Clone)]
enum Pipeline {
    Input,
    Spawner,
    AI,
    ShipMovement,
    CannonballMovement,
    LaserCleanup,
    HUD
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Ship {
    steering_wheel: SteeringWheel,
    health: i32
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
            -consts::TAU * 3.0, 
            consts::TAU * 3.0
        );
    }
}

#[derive(Component)]
struct Cannonball;

#[derive(Component)]
struct LaserGun {
    last_fired: f64
}

#[derive(Component)]
struct Laser {
    fired: f64
}

#[derive(Component)]
struct Spawner {
    last_spawned: f64,
    until_next: f64
}

struct EnemyCounter {
    to_spawn: i32,
    dead: i32
}

fn player_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    // Create the player ship
    commands.spawn_bundle(RigidBodyBundle {
        position: Vec3::new(0.0, 0.0, 0.0).into(),
        forces: RigidBodyForces {
            gravity_scale: 0.0,
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
        // Add laser gun
        let laser_t =
            Transform::from_translation(Vec3::new(1.5, 1.2, 0.0))
                .with_rotation(Quat::from_rotation_y(consts::FRAC_PI_2))
                .with_scale(Vec3::splat(6.0));
        ship.spawn_bundle(PbrBundle {
            transform: laser_t,
            ..Default::default()
        }).with_children(|laser| {
            // Add laser model
            laser.spawn_scene(
                asset_server.load("models/blasterG.glb#Scene0")
            );
        })
        .insert(LaserGun { last_fired: 0.0 });
    })
    .insert(Ship {
        steering_wheel: SteeringWheel {
            angle: 0.0,
        },
        health: 200
    })
    .insert(Player);
}

fn spawner_setup(
    mut commands: Commands
) {
    for x in [-40.0, 40.0] {
        for z in [-40.0, 40.0] {
            commands.spawn_bundle(PbrBundle {
                transform: Transform::from_translation(Vec3::new(x, 0.0, z)),
                ..Default::default()
            }).insert(Spawner { last_spawned: 0.0, until_next: 0.0} );
        }
    }
}

fn enemy_spawner(
    mut commands: Commands,
    mut spawners: Query<(&mut Spawner, &Transform)>,
    enemies: Query<&Ship, Without<Player>>,
    asset_server: Res<AssetServer>,
    mut enemy_counter: ResMut<EnemyCounter>,
    time: Res<Time>
) {
    if enemies.iter().count() >= 6 || enemy_counter.to_spawn <= 0 {
        return;
    }
    for (mut spawner, spawner_t) in spawners.iter_mut() {
        let now = time.seconds_since_startup();
        let since_last_spawn = now - spawner.last_spawned;
        if since_last_spawn > spawner.until_next {
            spawner.last_spawned = now;
            spawner.until_next = rand::random::<f64>() * 20.0 + 20.0;
            enemy_counter.to_spawn -= 1;
            // Create enemy entity
            commands.spawn_bundle(RigidBodyBundle {
                position: (
                    spawner_t.translation.clone(),
                    Quat::from_rotation_y(rand::random::<f32>() * consts::TAU)
                ).into(),
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
                health: 40
            }).insert(Cannon {
                last_fired: 0.0
            });
        }
    }
}

fn camera_setup(
    mut commands: Commands
) {
    let mut camera = OrthographicCameraBundle::new_3d();
    camera.orthographic_projection.scale = 20.0;
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
    button_inputs: Res<Input<GamepadButton>>,
    // button_axes: Res<Axis<GamepadButton>>,
    axes: Res<Axis<GamepadAxis>>,
    mut prev_input: ResMut<PreviousInput>,
    mut player_ships: Query<&mut Ship, With<Player>>,

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
                if delta_angle > consts::PI {
                    delta_angle - consts::TAU
                } else if delta_angle < -consts::PI {
                    delta_angle + consts::TAU
                } else {
                    delta_angle
                };
            
            player_ship.steering_wheel.turn(delta_angle);
            prev_input.angle = new_angle;
        }
    }
}

fn laser_gun_handler(
    mut commands: Commands,
    gamepads: Res<Gamepads>,
    button_axes: Res<Axis<GamepadButton>>,
    mut lasers: Query<(Entity, &mut LaserGun, &GlobalTransform)>,
    mut player_rb: Query<(
        &mut RigidBodyVelocityComponent,
        &RigidBodyMassPropsComponent
    ), With<Player>>,
    query_pipeline: Res<QueryPipeline>,
    collider_query: QueryPipelineColliderComponentsQuery,
    mut lines: ResMut<DebugLines>,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut enemy_counter: ResMut<EnemyCounter>,
    enemies: Query<&Ship, Without<Player>>
) {
    if let Some(gamepad) = gamepads.iter().next() {
        if let Some((laser_ent, mut laser_com, laser_t)) = lasers.iter_mut().next() {
            let now = time.seconds_since_startup();
            let right_trigger = button_axes
                .get(GamepadButton(*gamepad, GamepadButtonType::RightTrigger2))
                .unwrap();
            if right_trigger.abs() > 0.01 && now - laser_com.last_fired > LASER_COOLDOWN {
                laser_com.last_fired = now;
                // fire the laser
                let collider_set = QueryPipelineColliderComponentsSet(&collider_query);
                let shape = Ball::new(1.0);
                let shape_pos = (laser_t.translation + laser_t.forward()*-2.0, Quat::from_rotation_x(0.4)).into();
                let shape_vel = (laser_t.forward() * -1.0).into();
                let max_toi = 50.0;
                let groups = InteractionGroups::all();
                let filter = None;

                commands.entity(laser_ent).with_children(|parent| {
                    parent.spawn_bundle(PbrBundle {
                        transform: Transform::from_rotation(Quat::from_rotation_x(consts::FRAC_PI_2)),
                        mesh: meshes.add(Mesh::from(bevy::prelude::shape::Capsule {
                            radius: 0.1,
                            rings: 1,
                            depth: 49.0,
                            ..Default::default()
                        })),
                        material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
                        ..Default::default()
                    }).insert(Laser {fired: now});
                });

                // recoil
                if let Some((mut rbv, rbmp)) = player_rb.iter_mut().next() {
                    rbv.apply_impulse(rbmp, (laser_t.forward() * 10000.0).into());
                }

                if let Some((handle, hit)) = query_pipeline.cast_shape(
                    &collider_set, &shape_pos, &shape_vel, &shape, max_toi, groups, filter
                ) {
                    // The first collider hit has the handle `handle`. The `hit` is a
                    // structure containing details about the hit configuration.
                    println!("Hit the entity {:?} with the configuration: {:?}", handle.entity(), hit);
                    if let Ok(_) = enemies.get(handle.entity()) {
                        commands.entity(handle.entity()).despawn_recursive();
                        enemy_counter.dead += 1;
                    }
                }
            }
        }
    }
}

fn laser_cleanup(
    mut commands: Commands,
    time: Res<Time>,
    mut lasers: Query<(Entity, &Laser, &mut Transform)>
) {
    let now = time.seconds_since_startup();
    for (ent, laser, mut t) in lasers.iter_mut() {
        let since_fired = now - laser.fired;
        if since_fired > LASER_TIMEOUT {
            commands.entity(ent).despawn_recursive();
        } else {
            let girth = (since_fired / LASER_TIMEOUT) as f32;
            t.scale = Vec3::new(0.0f32.max(t.scale.x * girth), 1.0, 0.0f32.max(t.scale.x * girth));
        }
    }
}

fn ship_movement(
    mut ships: Query<(
        &Ship,
        &Transform,
        &mut RigidBodyForcesComponent,
        &mut RigidBodyMassPropsComponent
    )>,
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

fn enemy_movement_ai(
    mut commands: Commands,
    mut enemy_ships: Query<(Entity, &mut Ship, &Transform), Without<Player>>,
    mut player_ts: Query<&Transform, With<Player>>,
    mut lines: ResMut<DebugLines>,
    mut enemy_counter: ResMut<EnemyCounter>
) {
    // Try and move into range of the player
    if let Some(player_t) = player_ts.iter().next() {
        for (enemy_ent, mut enemy_ship, t) in enemy_ships.iter_mut() {
            if enemy_ship.health <= 0 {
                commands.entity(enemy_ent).despawn_recursive();
                enemy_counter.dead += 1;
            };
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
                // angle > consts::FRAC_PI_2 - 0.3 &&
                // angle < consts::FRAC_PI_2 + 0.3
            {
                if is_to_left_of_player(player_t, t) {
                    // fire to the left
                    fire_cannon(&mut commands, t, t.left(), &asset_server);
                } else {
                    // fire to the right
                    fire_cannon(&mut commands, t, t.right(), &asset_server);
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

fn hud_handler(
    mut text_query: Query<&mut Text, With<HUD>>,
    player: Query<&Ship, With<Player>>,
    enemy_counter: Res<EnemyCounter>
) {
    if let Some(player) = player.iter().next() {
        if let Some(mut text_box) = text_query.iter_mut().next() {
            text_box.sections[0].value = format!("health: {}\nenemies left: {}", player.health, ENEMY_COUNT - enemy_counter.dead);
        }
    }
}

// TODO
// - player laser
//  - hit multiple targets
// - boundaries on map
// - sound effects and music
// - game over screen
// - wheel animation

// -- submit --

// - simple visual effects
// - splash screen