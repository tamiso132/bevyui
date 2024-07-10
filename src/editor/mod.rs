use std::{
    cmp::{max, min},
    time::{Duration, Instant},
};

use ash::{
    valve::mutable_descriptor_type,
    vk::{self, Extent2D},
};
use bevy::{
    app::{Plugin, Startup, Update},
    prelude::{Commands, IntoSystemConfigs, NonSendMut, Res},
};
use imgui::ImguiApp;
use reflection::{Bar, EntitiesMeta, EntityMeta, Foo, ReflectionMarker};
use voxelengine::{
    vulkan::{util, VulkanContext},
    App::ApplicationTrait,
};
use winit::event_loop::{self, EventLoop};

mod imgui;
mod reflection;

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
        loop {
            context.run_non_block(&mut event_loop, &mut entities_meta, &mut entity);
        }
    }
}

fn create_event_loop() -> EventLoop<()> {
    EventLoop::new().unwrap()
}

fn test_spawn(mut commands: Commands) {
    // commands.spawn((Foo::default(), ReflectionMarker::default()));
    commands.spawn((Foo::default(), ReflectionMarker::default()));
    commands.spawn((Bar { b: 2, t: 7, bba: 15 }, ReflectionMarker::default()));
}

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let event_loop = create_event_loop();
        event_loop.set_control_flow(event_loop::ControlFlow::Wait);
        let imgui_app = ImguiApp::on_new(&event_loop);

        app.insert_non_send_resource(imgui_app);
        app.insert_non_send_resource(reflection::EntitiesMeta { data: vec![] });
        app.insert_non_send_resource(EntityMeta::default());
        app.insert_non_send_resource(event_loop);

        app.add_systems(Startup, (test_spawn, reflection::setup_reflection).chain());
        app.add_systems(
            Update,
            (reflection::parse_world_entities_data, EditorPlugin::update_imgui, reflection::write_out_data).chain(),
        );
    }
}
