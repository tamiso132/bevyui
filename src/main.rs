#![feature(vec_into_raw_parts)]
#![feature(duration_millis_float)]

use bevy::{
    app::{App, Last, Plugin, PostUpdate, Startup, Update},
    core::{FrameCountPlugin, TaskPoolPlugin},
    log::LogPlugin,
    math::Vec3,
    prelude::{default, Camera3dBundle, Commands, Component, Entity, EventReader, IntoSystemConfigs, NonSendMut, Query, Res, ResMut, Resource, With},
    time::{Time, TimePlugin},
    transform::components::Transform,
    window::{PrimaryWindow, Window, WindowPlugin},
    DefaultPlugins,
};
use bevy_framepace::Limiter;
use bevy_rapier2d::{
    plugin::{NoUserData, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};
use bevy_reflect::{Reflect, TypeRegistration};
use bevy_winit::{WinitPlugin, WinitWindows};
use editor::EditorPlugin;
use winit::{
    event::{self, WindowEvent},
    event_loop::{self, EventLoop, EventLoopBuilder},
    raw_window_handle::HasWindowHandle,
};

mod editor;
mod game;

#[derive(Resource, Default)]
pub struct Frames {
    total_frames: f32,
    total_delta: f32,
}
fn calculate_frame_avg() {}

pub fn frame_time(time: Res<Time>, mut frames: ResMut<Frames>) {
    frames.total_frames += 1.0;
    frames.total_delta += time.delta().as_millis_f32();

    let avg_time = frames.total_delta / frames.total_frames;

    println!("Time: {}", avg_time);
}

fn main() {
    let mut app = App::new();
    game::setup_game_systmes(&mut app);

    app.insert_resource(Frames::default());

    //app.add_plugins((DefaultPlugins)).add_systems(Update, editor::event::handle_all_events).run();
    app.add_plugins((
       // DefaultPlugins,
        RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0),
        RapierDebugRenderPlugin::default(),
        EditorPlugin,
    ))
    .run();
}
