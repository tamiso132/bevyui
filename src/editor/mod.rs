use std::time::{Duration, Instant};

use ash::vk;
use bevy::{
    app::{Plugin, Startup, Update},
    prelude::{AppTypeRegistry, Commands, IntoSystemConfigs, NonSendMut, Res, World},
};
use imgui::{CollapsingHeader, Condition};
use reflection::{EntitiesMeta, EntityMeta, Foo, ReflectionMarker};
use voxelengine::{
    vulkan::{util, VulkanContext},
    App::ApplicationTrait,
};
use winit::event_loop::{self, EventLoop};

mod reflection;

pub struct ImguiApp {
    vulkan: VulkanContext,
    resize: bool,
    last_frame: Instant,

    entities: Vec<EntityMeta>,

    test: bool,
    window: bool,
}

const HZ_MAX: i64 = (1000.0 / 60.0) as i64;

impl ApplicationTrait for ImguiApp {
    fn on_new(event_loop: &EventLoop<()>) -> Self {
        let vulkan = VulkanContext::new(event_loop, 2, true);

        Self { vulkan, last_frame: Instant::now(), resize: false, entities: vec![], test: false, window: true }
    }

    fn on_new_frame(&mut self, event: &winit::event::Event<()>) {
        let now = Instant::now();
        let delta_time = now.duration_since(self.last_frame);

        // TODO, make this into 1 update function, no reason to have them seperate
        self.vulkan.imgui.as_mut().unwrap().update_delta_time(delta_time);
        self.last_frame = now;
        self.vulkan.imgui.as_mut().unwrap().process_event_imgui(&self.vulkan.window, &event);
    }

    fn on_draw(&mut self) {
        self.vulkan.prepare_frame(&mut self.resize);

        if self.resize {
            unsafe {
                self.vulkan.device.device_wait_idle().unwrap();
                self.vulkan.recreate_swapchain();
                self.vulkan.prepare_frame(&mut self.resize);
                self.resize = false;
            }
        }
        let frame_index = self.vulkan.current_frame;
        let cmd = self.vulkan.cmds[frame_index];
        let device = &self.vulkan.device;
        let set = self.vulkan.resources.set;

        util::transition_image_color(device, cmd, self.vulkan.get_swapchain_image().image);

        self.vulkan.begin_rendering(vk::AttachmentLoadOp::CLEAR);
        self.vulkan.end_rendering();
        let imgui = self.vulkan.imgui.as_mut().unwrap();
        let mut ui = imgui.get_draw_instance(&self.vulkan.window);

        // draw imgui
        let w = ui
            .window("Collapsing header")
            .opened(&mut self.window)
            .resizable(true)
            .position([20.0, 20.0], Condition::Appearing)
            .size([700.0, 500.0], Condition::Appearing);
        w.build(|| {
            if CollapsingHeader::new("I'm a collapsing header. Click me!").build(ui) {
                ui.text("A collapsing header can be used to toggle rendering of a group of widgets");
            }

            ui.spacing();
            if CollapsingHeader::new("I'm open by default").default_open(true).build(ui) {
                ui.text("You can still close me with a click!");
            }

            ui.spacing();
            if CollapsingHeader::new("I only open with double-click")
                .open_on_double_click(true)
                .build(ui)
            {
                ui.text("Double the clicks, double the fun!");
            }

            ui.spacing();
            if CollapsingHeader::new("I don't have an arrow").bullet(true).build(ui) {
                ui.text("Collapsing headers can use a bullet instead of an arrow");
            }

            ui.spacing();
            if CollapsingHeader::new("I only open if you click the arrow").open_on_arrow(true).build(ui) {
                ui.text("You clicked the arrow");
            }

            ui.spacing();
            ui.checkbox("Toggle rendering of the next example", &mut self.test);
            if CollapsingHeader::new("I've got a separate close button").build_with_close_button(ui, &mut self.test) {
                ui.text("I've got contents just like any other collapsing header");
            }
        });
        // DRAW HIERACHY
        // for entity in &mut self.entities {
        //     u1.text(entity.id.to_string());
        // }
        imgui.render(
            self.vulkan.window_extent,
            &self.vulkan.swapchain.images[self.vulkan.swapchain.image_index as usize],
            self.vulkan.current_frame,
            &mut self.vulkan.resources,
            cmd,
            set,
        );

        if self.vulkan.end_frame_and_submit() {
            self.resize = true;
        }

        let now = Instant::now();

        let diff = now - self.last_frame;

        let hz_diff = HZ_MAX - diff.as_millis() as i64;

        if hz_diff > 0 {
            std::thread::sleep(Duration::from_millis(hz_diff as u64));
        }

        self.vulkan.window.request_redraw();
    }

    fn on_mouse_motion(&mut self, delta: &(f64, f64)) {}

    fn on_key_press(&mut self, key_event: &winit::event::RawKeyEvent, keycode: winit::keyboard::KeyCode) {}

    fn on_destroy(&mut self) {
        unsafe {
            self.vulkan.device.device_wait_idle().unwrap();
        }
        self.vulkan.destroy();
    }

    fn resize_event(&mut self) {
        self.resize = true;
    }

    fn set_imgui_draw(&mut self, imgui_func: fn(ui: &mut imgui::Ui)) {}
}

pub struct EditorPlugin;

impl EditorPlugin {
    fn update_imgui(
        mut context: NonSendMut<voxelengine::App::App<ImguiApp>>,
        mut event_loop: NonSendMut<EventLoop<()>>,
        entities_meta: Res<EntitiesMeta>,
    ) {
        let diff = entities_meta.data.len() - context.application.entities.len();
        if diff > 0 {
            for _ in 0..diff {
                context.application.entities.push(EntityMeta::default());
            }
        }

        unsafe {
            let src_ptr = entities_meta.data.as_ptr();
            let dst_ptr = context.application.entities.as_mut_ptr();

            std::ptr::copy_nonoverlapping(src_ptr, dst_ptr, entities_meta.data.len());
        }
        context.run_non_block(&mut event_loop);
    }
}

fn create_event_loop() -> EventLoop<()> {
    EventLoop::new().unwrap()
}

fn test_spawn(mut commands: Commands) {
    commands.spawn((Foo::default(), ReflectionMarker::default()));
    commands.spawn((Foo::default(), ReflectionMarker::default()));
    commands.spawn((Foo::default(), ReflectionMarker::default()));
    commands.spawn((Foo::default(), ReflectionMarker::default()));
}

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let event_loop = create_event_loop();
        event_loop.set_control_flow(event_loop::ControlFlow::Wait);
        let mut imgui_app: voxelengine::App::App<ImguiApp> = voxelengine::App::App::new(&event_loop);

        imgui_app.run(event_loop);
        todo!();
        app.insert_non_send_resource(imgui_app);
        app.insert_non_send_resource(event_loop);
        app.insert_resource(reflection::EntitiesMeta { data: vec![] });

        app.add_systems(Startup, (test_spawn, reflection::setup_reflection));
        app.add_systems(Update, (reflection::parse_world_entities_data, EditorPlugin::update_imgui));
    }
}
