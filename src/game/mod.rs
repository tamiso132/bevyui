use bevy::{
    app::{App, Startup, Update},
    asset::{AssetServer, Assets},
    input::ButtonInput,
    math::Vec3,
    prelude::{Camera2dBundle, Commands, Component, KeyCode, Query, Res, ResMut, With},
    render::mesh::Mesh,
    sprite::{ColorMaterial, SpriteBundle},
    time::Time,
    transform::components::Transform,
    utils::default,
};

pub struct Position {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}

pub fn setup_game_systmes(app: &mut App) {
    app.add_systems(Startup, setup_game);
    app.add_systems(Update, sprite_movement);
}

#[derive(Component)]
struct Player;

#[derive(Component)]
enum Direction {
    Up,
    Down,
}

fn setup_game(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Load the Bevy logo as a texture
    let texture_handle = asset_server.load("player.png");
    // Build a default quad mesh

    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        SpriteBundle { texture: texture_handle, transform: Transform::from_xyz(100., 0., 0.), ..default() },
        Direction::Up,
    ));
}

fn player_movement(keyboard_input: Res<ButtonInput<KeyCode>>, mut query: Query<&mut Transform, With<Player>>) {
    for mut transform in query.iter_mut() {
        let mut direction = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::KeyW) {
            direction.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            direction.y -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }

        transform.translation += direction * 2.0; // Adjust speed if needed
    }
}

fn sprite_movement(time: Res<Time>, mut sprite_position: Query<(&mut Direction, &mut Transform)>) {
    for (mut logo, mut transform) in &mut sprite_position {
        match *logo {
            Direction::Up => transform.translation.y += 150. * time.delta_seconds(),
            Direction::Down => transform.translation.y -= 150. * time.delta_seconds(),
        }

        if transform.translation.y > 200. {
            *logo = Direction::Down;
        } else if transform.translation.y < -200. {
            *logo = Direction::Up;
        }
    }
}
