use std::{
    any::Any,
    ops::Deref,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use ash::vk;
use bevy::{
    app::{AppExit, Plugin, Startup, Update},
    prelude::{EventWriter, NonSend, NonSendMut, Res, ResMut, Resource, World},
};
use bevy_winit::WinitWindows;
use voxelengine::{
    core::camera::Camera,
    vulkan::{resource::AllocatedBuffer, util, VulkanContext},
    App::{App, ApplicationTrait},
};
use winit::{
    event_loop::{self, EventLoop},
    platform::{pump_events::EventLoopExtPumpEvents, run_on_demand::EventLoopExtRunOnDemand},
};

pub struct ImguiApp {
    vulkan: VulkanContext,
    resize: bool,
    last_frame: Instant,
}

const HZ_MAX: i64 = (1000.0 / 60.0) as i64;

impl ApplicationTrait for ImguiApp {
    fn on_new(event_loop: &EventLoop<()>) -> Self {
        let vulkan = VulkanContext::new(event_loop, 2, true);

        Self { vulkan, last_frame: Instant::now(), resize: false }
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
        let u1 = imgui.get_draw_instance(&self.vulkan.window);
        u1.show_demo_window(&mut true);

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
}

pub struct EditorPlugin;

impl EditorPlugin {
    fn update_imgui(mut context: NonSendMut<voxelengine::App::App<ImguiApp>>, mut event_loop: NonSendMut<EventLoop<()>>) {
        context.run_non_block(&mut event_loop);
    }
}

fn create_event_loop() -> EventLoop<()> {
    EventLoop::new().unwrap()
}
#[derive(Resource)]
struct ImguiClose(Arc<Mutex<bool>>);

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let event_loop = create_event_loop();
        event_loop.set_control_flow(event_loop::ControlFlow::Wait);
        let imgui_app: voxelengine::App::App<ImguiApp> = voxelengine::App::App::new(&event_loop);

        app.insert_non_send_resource(imgui_app);
        app.insert_non_send_resource(event_loop);

        app.add_systems(Update, (EditorPlugin::update_imgui));
    }
}
