#![feature(vec_into_raw_parts)]
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use bevy::{
    app::{App, Plugin, Startup, Update},
    core::{FrameCountPlugin, TaskPoolPlugin},
    log::LogPlugin,
    math::Vec3,
    prelude::{default, Camera3dBundle, Commands, Component, IntoSystemConfigs, Query, Res, ResMut, With},
    time::TimePlugin,
    transform::components::Transform,
    window::{Window, WindowPlugin},
    DefaultPlugins,
};
use bevy_framepace::Limiter;
use bevy_reflect::{Reflect, TypeRegistration};
use bevy_winit::WinitPlugin;
use editor::EditorPlugin;
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

mod editor;
mod game;

fn startup(mut r: ResMut<bevy_framepace::FramepaceSettings>) {
    r.limiter = Limiter::from_framerate(60.0);
}

fn main() {
    let mut app = App::new();
    game::setup_game_systmes(&mut app);
    // app.add_plugins((DefaultPlugins, EditorPlugin, bevy_framepace::FramepacePlugin))
    //     .add_systems(Startup, (startup))
    //     .run();

    app.add_plugins(DefaultPlugins).run();
}
