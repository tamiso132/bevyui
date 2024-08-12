use std::{
    cmp::{max, min},
    ops::Deref,
    time::{Duration, Instant},
};

pub mod event;

use ash::{
    valve::mutable_descriptor_type,
    vk::{self, Extent2D},
};
use bevy::{
    app::{Plugin, Startup, Update},
    prelude::*,
};
use bevy_winit::WinitWindows;
use imgui::ImguiApp;
use reflection::{Bar, EntitiesMeta, EntityMeta, Foo};
use voxelengine::vulkan::{util, VulkanContext};
use winit::{
    event_loop::{self, EventLoop},
    raw_window_handle::HasWindowHandle,
};

mod imgui;
mod reflection;
mod structs;

#[derive(Component, Default)]
pub struct ReflectionMarker;

pub struct EditorPlugin;

impl EditorPlugin {
    fn update_imgui(
        mut context: NonSendMut<ImguiApp>,
        mut event_loop: NonSendMut<EventLoop<()>>,
        mut entities_meta: NonSendMut<EntitiesMeta>,
        mut entity: NonSendMut<EntityMeta>,
    ) {
        // let diff = entities_meta.data.len() - context.entities.len();
        // if diff > 0 {
        //     for _ in 0..diff {
        //         context.entities.push(EntityMeta::default());
        //     }
        // }

        // unsafe {
        //     let src_ptr = entities_meta.data.as_ptr();
        //     let dst_ptr = context.application.entities.as_mut_ptr();

        //     std::ptr::copy_nonoverlapping(src_ptr, dst_ptr, entities_meta.data.len());
        // }
        context.run_non_block(&mut event_loop, &mut entities_meta, &mut entity);
    }
}

fn create_event_loop() -> EventLoop<()> {
    EventLoop::new().unwrap()
}

fn test_spawn(mut commands: Commands) {
    // commands.spawn((Foo::default(), ReflectionMarker::default()));
    let one = commands.spawn((Foo::default(), ReflectionMarker::default(), Transform::default()));
    let two = commands.spawn((Bar { b: 2, t: 7, bba: "test_string".to_string(), l: 10 }, ReflectionMarker::default()));
    let x = 5;
}

fn setup(world: &mut World) {
    let event_loop = create_event_loop();
    event_loop.set_control_flow(event_loop::ControlFlow::Wait);

    let imgui_app = ImguiApp::on_new(&event_loop);

    world.insert_non_send_resource(imgui_app);
    world.insert_non_send_resource(reflection::EntitiesMeta { data: vec![] });
    world.insert_non_send_resource(EntityMeta::default());
    world.insert_non_send_resource(event_loop);
}

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, (setup, test_spawn, reflection::setup_reflection).chain());
        app.add_systems(
            Update,
            (
                reflection::parse_world_entities_data,
                EditorPlugin::update_imgui,
                reflection::mutate_data,
                reflection::write_out_data,
            )
                .chain(),
        );
    }
}
