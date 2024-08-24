use std::{
    cmp::{max, min},
    ops::Deref,
    time::{Duration, Instant},
};

use ash::{
    valve::mutable_descriptor_type,
    vk::{self, Extent2D},
};
use bevy::{
    app::{MainScheduleOrder, Plugin, Startup, Update},
    ecs::schedule::ScheduleLabel,
    prelude::*,
    time::common_conditions::on_timer,
};
use bevy_winit::WinitWindows;
use imgui::ImguiApp;
use reflection::{Bar, EntitiesMeta, EntityMeta, Foo};
use voxelengine::vulkan::{util, VulkanContext};
use winit::{
    event_loop::{self, EventLoop},
    raw_window_handle::HasWindowHandle,
};

mod display;
mod imgui;
mod reflection;
mod structs;
mod t_reflect;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct PrepareUpdate;

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

    let type_registry = world.get_resource::<AppTypeRegistry>().unwrap();

    // REFLECTION
    t_reflect::register_bevy_types(type_registry);

    // REMOVE IF NOT USED
   // Bar::register(type_registry);
   // Foo::register(type_registry);
}
pub fn reveal_map(mut query: Query<&mut Visibility>) {
    for mut i in query.iter_mut() {
        if *i == Visibility::Visible {
            //    *i = Visibility::Hidden;
        }
    }
}

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_schedule(PrepareUpdate);

        app.world_mut().resource_mut::<MainScheduleOrder>().insert_after(PostUpdate, PreUpdate);

        app.add_systems(Startup, (setup, test_spawn).chain());
        app.add_systems(
            PreUpdate,
            (
                reflection::parse_world_entities_data,
                EditorPlugin::update_imgui,
                reflection::mutate_data,
                reflection::check_component_delete,
                reveal_map,
            )
                .chain()
                .run_if(on_timer(Duration::from_millis(30))),
        );
    }
}
