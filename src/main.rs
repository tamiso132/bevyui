#![feature(vec_into_raw_parts)]

use bevy::{
    app::{App, Startup},
    prelude::{Component, ResMut},
    DefaultPlugins,
};
use bevy_framepace::Limiter;
use bevy_reflect::Reflect;
use editor::EditorPlugin;

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
    app.add_plugins((DefaultPlugins, EditorPlugin, bevy_framepace::FramepacePlugin))
        .add_systems(Startup, startup)
        .run();

    // app.add_plugins(DefaultPlugins).run();
}
