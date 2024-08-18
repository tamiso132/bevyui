use bevy::{
    app::{App, Startup, Update},
    asset::{AssetServer, Assets, Handle},
    color::{ColorToComponents, LinearRgba},
    input::ButtonInput,
    math::{Vec2, Vec3, VectorSpace},
    prelude::{Camera2dBundle, Commands, Component, IntoSystemConfigs, IntoSystemSet, KeyCode, Query, Rectangle, Res, ResMut, With},
    render::mesh::Mesh,
    sprite::{ColorMaterial, SpriteBundle},
    time::Time,
    transform::components::Transform,
    utils::default,
};
use bevy_rapier2d::{
    plugin::RapierConfiguration,
    prelude::{Collider, LockedAxes, RigidBody},
};

pub const OBJECT_SIZE: f32 = 32.0;
mod spawn;

#[repr(C)]
#[derive(Clone, Copy, Component)]
pub struct Position {
    pub x: isize,
    pub y: isize,
}
impl Position {
    pub fn distance(&self, other: Position, threshold: f32) -> bool {
        let distance = (((self.x - other.x).pow(2) + (self.y - other.y).pow(2)) as f32).powf(0.5);

        distance < threshold
    }
}

#[derive(Clone, Copy, Component)]
pub struct Size {
    pub x: usize,
    pub y: usize,
}

#[repr(C)]
#[derive(Clone, Copy, Component)]
pub struct Crate {
    pub goal_crate: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Component)]
pub struct Wall;

pub fn setup_game_systmes(app: &mut App) {
    let b = setup_game;
    b.into_system_set();
    app.add_systems(Startup, setup_game);
    app.add_systems(Update, (sprite_movement, update_position, check_win_conditions).chain());
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Goal;

#[derive(Component)]
enum Direction {
    Up,
    Down,
    Right,
    Left,
}

fn setup_game(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut config: ResMut<RapierConfiguration>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Load the Bevy logo as a texture
    let texture_handle = asset_server.load("player.png");
    let texture_wall = asset_server.load("wall.png");
    let texture_crate = asset_server.load("crate.png");
    let texture_regular_crate = asset_server.load("cratemarked.png");
    let texture_blank = asset_server.load("blank.png");
    let texture_goal = asset_server.load("blankmarked.png");
    // Build a default quad mesh

    config.gravity = Vec2::ZERO;
    commands.spawn(Camera2dBundle::default());
    commands
        .spawn((
            SpriteBundle { texture: texture_handle, transform: Transform::from_xyz(0., 0., 0.), ..default() },
            Direction::Up,
            Collider::cuboid(32.0 / 2.0, 32.0 / 2.0),
            RigidBody::Dynamic,
        ))
        .insert(LockedAxes::ROTATION_LOCKED);

    commands.spawn(
        (SpriteBundle {
            texture: texture_blank,
            transform: Transform { translation: Vec3::new(0.0, 0.0, -2.0), scale: Vec3::new(800.0, 600.0, 1.0), ..Default::default() },
            ..Default::default()
        }),
    );

    spawn::spawn_wall(&mut commands, Position { x: -2, y: 0 }, texture_wall.clone());
    spawn::spawn_wall(&mut commands, Position { x: -3, y: 0 }, texture_wall.clone());
    spawn::spawn_wall(&mut commands, Position { x: -4, y: 0 }, texture_wall.clone());

    spawn::spawn_wall(&mut commands, Position { x: -4, y: 1 }, texture_wall.clone());
    spawn::spawn_wall(&mut commands, Position { x: -4, y: 2 }, texture_wall.clone());

    spawn::spawn_wall(&mut commands, Position { x: -3, y: 2 }, texture_wall.clone());
    spawn::spawn_wall(&mut commands, Position { x: -2, y: 2 }, texture_wall.clone());

    spawn::spawn_wall(&mut commands, Position { x: -2, y: 3 }, texture_wall.clone());

    spawn::spawn_wall(&mut commands, Position { x: -1, y: 3 }, texture_wall.clone());
    spawn::spawn_wall(&mut commands, Position { x: 0, y: 3 }, texture_wall.clone());
    spawn::spawn_wall(&mut commands, Position { x: 1, y: 3 }, texture_wall.clone());
    spawn::spawn_wall(&mut commands, Position { x: 2, y: 3 }, texture_wall.clone());

    spawn::spawn_wall(&mut commands, Position { x: 2, y: 2 }, texture_wall.clone());
    spawn::spawn_wall(&mut commands, Position { x: 2, y: 1 }, texture_wall.clone());
    spawn::spawn_wall(&mut commands, Position { x: 2, y: 0 }, texture_wall.clone());
    spawn::spawn_wall(&mut commands, Position { x: 2, y: -1 }, texture_wall.clone());
    spawn::spawn_wall(&mut commands, Position { x: 2, y: -2 }, texture_wall.clone());
    spawn::spawn_wall(&mut commands, Position { x: 2, y: -3 }, texture_wall.clone());

    spawn::spawn_wall(&mut commands, Position { x: 3, y: -3 }, texture_wall.clone());

    spawn::spawn_wall(&mut commands, Position { x: 3, y: -4 }, texture_wall.clone());
    spawn::spawn_wall(&mut commands, Position { x: 3, y: -5 }, texture_wall.clone());
    spawn::spawn_wall(&mut commands, Position { x: 3, y: -6 }, texture_wall.clone());

    spawn::spawn_wall(&mut commands, Position { x: 2, y: -6 }, texture_wall.clone());
    spawn::spawn_wall(&mut commands, Position { x: 1, y: -6 }, texture_wall.clone());
    spawn::spawn_wall(&mut commands, Position { x: 0, y: -6 }, texture_wall.clone());
    spawn::spawn_wall(&mut commands, Position { x: -1, y: -6 }, texture_wall.clone());
    spawn::spawn_wall(&mut commands, Position { x: -2, y: -6 }, texture_wall.clone());
    spawn::spawn_wall(&mut commands, Position { x: -3, y: -6 }, texture_wall.clone());
    spawn::spawn_wall(&mut commands, Position { x: -4, y: -6 }, texture_wall.clone());

    spawn::spawn_wall(&mut commands, Position { x: -4, y: -5 }, texture_wall.clone());
    spawn::spawn_wall(&mut commands, Position { x: -4, y: -4 }, texture_wall.clone());
    spawn::spawn_wall(&mut commands, Position { x: -4, y: -3 }, texture_wall.clone());
    spawn::spawn_wall(&mut commands, Position { x: -4, y: -2 }, texture_wall.clone());
    spawn::spawn_wall(&mut commands, Position { x: -4, y: -1 }, texture_wall.clone());
    spawn::spawn_wall(&mut commands, Position { x: -4, y: 0 }, texture_wall.clone());

    // DETAILS
    spawn::spawn_wall(&mut commands, Position { x: -1, y: -1 }, texture_wall.clone());
    spawn::spawn_wall(&mut commands, Position { x: -2, y: -1 }, texture_wall.clone());
    spawn::spawn_wall(&mut commands, Position { x: -2, y: -2 }, texture_wall.clone());
    spawn::spawn_wall(&mut commands, Position { x: -2, y: -3 }, texture_wall.clone());

    // CRATES
    spawn::spawn_crate(&mut commands, Position { x: 1, y: 0 }, texture_regular_crate.clone());
    spawn::spawn_crate(&mut commands, Position { x: -3, y: 1 }, texture_regular_crate.clone());
    spawn::spawn_crate(&mut commands, Position { x: 0, y: -5 }, texture_regular_crate.clone());
    spawn::spawn_crate(&mut commands, Position { x: -1, y: -4 }, texture_regular_crate.clone());
    spawn::spawn_crate(&mut commands, Position { x: 2, y: -4 }, texture_regular_crate.clone());

    // CRATES GOAL
    spawn::spawn_crate_goal(&mut commands, Position { x: 0, y: -1 }, texture_crate.clone());
    spawn::spawn_crate_goal(&mut commands, Position { x: -3, y: -3 }, texture_crate.clone());

    spawn::spawn_goal(&mut commands, Position { x: 0, y: -2 }, texture_goal.clone());
    spawn::spawn_goal(&mut commands, Position { x: -3, y: -1 }, texture_goal.clone());
}

fn check_win_conditions(mut goal_positions: Query<&Position, With<Goal>>, mut all_crates: Query<(&Crate, &Position)>) {
    for goal in goal_positions.iter() {
        let mut is_goal = false;
        for c in all_crates.iter() {
            if !c.0.goal_crate {
                continue;
            }

            if c.1.distance(*goal, 32.0) {
                is_goal = true;
                break;
            }
        }
        if !is_goal {
            return;
        }
    }

    // WON
    println!("YOU WON");
}

fn sprite_movement(keyboard_input: Res<ButtonInput<KeyCode>>, time: Res<Time>, mut sprite_position: Query<(&mut Direction, &mut Transform)>) {
    if keyboard_input.pressed(KeyCode::KeyW) {
        *sprite_position.single_mut().0 = Direction::Up;
    } else if keyboard_input.pressed(KeyCode::KeyS) {
        *sprite_position.single_mut().0 = Direction::Down;
    } else if keyboard_input.pressed(KeyCode::KeyD) {
        *sprite_position.single_mut().0 = Direction::Right;
    } else if keyboard_input.pressed(KeyCode::KeyA) {
        *sprite_position.single_mut().0 = Direction::Left;
    } else {
        return;
    }

    for (logo, mut transform) in &mut sprite_position {
        match *logo {
            Direction::Up => transform.translation.y += 150. * time.delta_seconds(),
            Direction::Down => transform.translation.y -= 150. * time.delta_seconds(),
            Direction::Right => transform.translation.x += 150. * time.delta_seconds(),
            Direction::Left => transform.translation.x -= 150. * time.delta_seconds(),
        }
    }
}

fn update_position(mut sprite_position: Query<(&mut Position, &mut Transform)>) {
    for mut s in sprite_position.iter_mut() {
        s.0.x = s.1.translation.x as isize;
        s.0.y = s.1.translation.y as isize;
    }
}
