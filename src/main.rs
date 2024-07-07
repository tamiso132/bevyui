use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use appliction::{EditorPlugin, ImguiApp};
use bevy::{
    app::{App, Plugin, Startup, Update},
    math::Vec3,
    prelude::{default, Camera3dBundle, Commands, Component, IntoSystemConfigs, Query, Res, With},
    transform::components::Transform,
    window::Window,
    DefaultPlugins,
};
use winit::{
    event_loop::{self, EventLoop, EventLoopBuilder},
    platform::x11::EventLoopBuilderExtX11,
};

#[derive(Component)]
struct Position {
    x: f32,
    y: f32,
}
#[derive(Component)]
struct Person;

#[derive(Component)]
struct Name(String);

mod appliction;
#[derive(Component)]
struct MyCameraMarker;
fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle { transform: Transform::from_xyz(10.0, 12.0, 16.0).looking_at(Vec3::ZERO, Vec3::Y), ..default() },
        MyCameraMarker,
    ));
}

// TODO, create one for wayland and windows



fn main() {
    let mut app = App::new();
    let mut bevy_closed = Arc::new(Mutex::new(true));
    let mut imgui_closed = Arc::new(Mutex::new(true));
    
    // println!("hello muta");
    app.add_plugins((EditorPlugin, DefaultPlugins))
        .add_systems(Startup, (add_people, setup_scene, setup_camera))
        .add_systems(Update, ((print_position_system, greet_people)).chain())
        .run();
}

fn add_people(mut commands: Commands) {
    // commands.spawn((Person, Name("Elaina Proctor".to_string())));
    // commands.spawn((Person, Name("Renzo Hume".to_string())));
    // commands.spawn((Person, Name("Zayna Nieves".to_string())));
}

fn print_position_system(query: Query<&Position>) {
    for position in &query {
        println!("position: {} {}", position.x, position.y);
    }
}

fn greet_people(query: Query<&Name, With<Person>>) {
    for name in &query {
        println!("hello {}!", name.0);
    }
}

fn setup_scene(mut commands: Commands) {
    // add entities to the world
    // Spawn a second window
}
