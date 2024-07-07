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
use bevy_reflect::Reflect;
use reflection::{list_all_components, setup_entities, setup_reflection, Bar};
use winit::{
    event_loop::{self, EventLoop, EventLoopBuilder},
    platform::x11::EventLoopBuilderExtX11,
};

#[derive(Reflect, Component)]
struct Position {
    x: f32,
    y: f32,
}
#[derive(Component)]
struct Person;

#[derive(Component)]
struct Name(String);

mod appliction;
mod reflection;

// TODO, create one for wayland and windows

fn main() {
    let mut app = App::new();
    app.add_plugins(EditorPlugin)
        .add_systems(Startup, (setup_reflection, setup_entities, list_all_components).chain())
        .run();

    // println!("hello muta");
}
