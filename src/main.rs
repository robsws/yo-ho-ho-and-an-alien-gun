use bevy::prelude::*;

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
        .add_startup_system(setup_camera)
        .add_startup_system(spawn_player)
        .run();
}

fn setup_camera(
    mut commands: Commands
) {
    let mut camera = OrthographicCameraBundle::new_3d();
    camera.orthographic_projection.scale = 15.0;
    camera.transform = Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y);
    commands.spawn_bundle(camera);
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(4.0, 5.0, 4.0),
        ..Default::default()
    });
}

fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    commands.spawn_bundle(PbrBundle {
        ..Default::default()
    }).with_children(|parent| {
        parent.spawn_scene(
            asset_server.load("models/ship_light.gltf#Scene0")
        );
    });
}