#![feature(vec_into_raw_parts)]

use bevy::{
    app::{App, Plugin, Startup, Update},
    core::{FrameCountPlugin, TaskPoolPlugin},
    log::LogPlugin,
    math::Vec3,
    prelude::{default, Camera3dBundle, Commands, Component, Entity, EventReader, IntoSystemConfigs, NonSendMut, Query, Res, ResMut, With},
    time::TimePlugin,
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
    platform::x11::EventLoopBuilderExtX11,
    raw_window_handle::HasWindowHandle,
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
    //app.add_plugins((DefaultPlugins)).add_systems(Update, editor::event::handle_all_events).run();
    app.add_plugins((
        DefaultPlugins,
        RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0),
        RapierDebugRenderPlugin::default(),
        bevy_framepace::FramepacePlugin,
        EditorPlugin,
    ))
    .add_systems(Startup, (startup))
    .run();

    // app.add_plugins(DefaultPlugins).run();
}

// fn handle_winit_events(mut events: EventReader<WindowEvent>) {
//     winit::event::Event::<()>;
//     for event in events.iter() {
//         match event {
//             WindowEvent::Resized(size) => {
//                 println!("Window resized: {:?}", size);
//             }
//             WindowEvent::Moved(pos) => {
//                 println!("Window moved to: {:?}", pos);
//             }
//             WindowEvent::CloseRequested => {
//                 println!("Window close requested");
//             }
//             _ => (),
//         }
//     }
// }
