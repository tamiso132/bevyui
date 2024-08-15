use ash::vk::{self, Extent2D};
use bevy::{app::Plugin, prelude::NonSendMut};
use imgui::{Condition, Ui};
use lazy_static::lazy_static;
use once_cell::unsync::Lazy;
use reflection::{EntitiesMeta, EntityMeta, Foo};
use std::{
    cmp::min,
    collections::HashMap,
    mem::{transmute, ManuallyDrop},
    ops::{Add, DerefMut},
    ptr::copy_nonoverlapping,
    time::{Duration, Instant},
    u8,
};
use voxelengine::vulkan::{util, VulkanContext};
use winit::{
    dpi::Pixel,
    event::{self, Event, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
    platform::{pump_events::EventLoopExtPumpEvents, run_on_demand::EventLoopExtRunOnDemand},
};

use super::reflection::{self};

// you can do this, becuase you know the aligment is always a power of two
/// Returns the aligned ptr and the amount it padded;
pub fn align_ptr(ptr: *mut u8, aligment: usize) -> *mut u8 {
    let ptr = ptr as usize;
    let aligned_ptr = (ptr + aligment - 1) & !(aligment - 1);

    assert!(aligned_ptr == ptr);

    aligned_ptr as *mut u8
}

struct HierachyWindow {
    window: bool,
    width: f32,
    max_width: f32,
}

impl HierachyWindow {
    pub fn new() -> Self {
        Self { window: true, width: 0.2, max_width: (200.0) }
    }

    fn draw_hierachy<'a>(
        &mut self,
        ui: &mut Ui,
        hdpi_scale: f32,
        window_extent: Extent2D,
        entities: &'a mut NonSendMut<EntitiesMeta>,
    ) -> Option<EntityMeta> {
        let extent = window_extent;
        let inspector_width = window_extent.width as f32 * 0.5;

        let mut entity_pressed = None;
        let window_width = min(self.max_width as u32, inspector_width as u32) as f32;

        let w = ui
            .window("Hierachy")
            .opened(&mut self.window)
            .position([0.0, 0.0], Condition::Always)
            .size([window_width, extent.height as f32], Condition::Always);

        w.build(|| {
            let mut index: i32 = -1;
            for i in 0..entities.data.len() {
                let entity = &mut entities.data[i];
                let s = format!("{}", entity.id.index());
                if ui.button(s) {
                    index = i as i32;
                }
                ui.spacing();
            }
            if index >= 0 {
                entity_pressed = Some(entities.data[index as usize].clone());
            }
        });
        entity_pressed
    }
}

struct InspectorWindow {
    window: bool,
    width: f32,
    max_width: f32,

    active_entity: Option<EntityMeta>,
}

impl InspectorWindow {
    fn new() -> Self {
        Self { window: true, width: 0.4, max_width: 300.0, active_entity: None }
    }
    fn draw_inspector(&mut self, ui: &mut Ui, hdpi_scale: f32, window_extent: Extent2D, entity: Option<EntityMeta>) -> &mut Option<EntityMeta> {
        let extent = window_extent;
        let window_width = window_extent.width as f32 * 0.2;

        //   let window_width = min(self.max_width as u32, inspector_width as u32) as f32;

        // Get a raw pointer to the original `Ui`
        let ui_ptr: *mut Ui = ui as *mut Ui;

        ui.window("inspector")
            .opened(&mut self.window)
            .position([extent.width as f32 - window_width, 0.0], Condition::Always)
            .size([window_width, extent.height as f32], Condition::Always)
            .build(|| {
                if entity.is_some() {
                    let entity_meta = entity.unwrap();
                    let comp_len = entity_meta.components.len();
                    self.active_entity = Some(entity_meta);
                }

                if self.active_entity.is_some() {
                    let entity_meta = &mut self.active_entity.as_mut().unwrap();

                    let c = format!("Entity: {}", entity_meta.id.index());
                    ui.text(c.as_str());

                    for i in 0..entity_meta.components.len() {
                        let component = &mut entity_meta.components[i];
                        let mut b = !component.is_removed;
                        if imgui::CollapsingHeader::new(&component.name).build_with_close_button(ui, &mut b) {
                            component.reflect.display_imgui(&mut component.data, ui_ptr);
                            ui.spacing();
                        }
                        component.is_removed = !b;
                    }
                }
            });
        &mut self.active_entity
    }
}

pub struct ImguiApp {
    vulkan: VulkanContext,
    resize: bool,
    last_frame: Instant,

    inspector: InspectorWindow,
    hierachy: HierachyWindow,

    exit: bool,
}

const HZ_MAX: i64 = (1000.0 / 60.0) as i64;

impl ImguiApp {
    pub fn run_non_block(
        &mut self,
        event_loop: &mut EventLoop<()>,
        mut entities_meta: &mut NonSendMut<EntitiesMeta>,
        entity: &mut NonSendMut<EntityMeta>,
    ) {
        if !self.exit {
            event_loop.pump_events(Some(Duration::ZERO), |event, _control_flow| {
                self.run_event(event, _control_flow, &mut entities_meta, entity);
            });

            if self.exit {
                self.on_destroy();
                event_loop
                    .run_on_demand(move |_, control_flow| {
                        control_flow.exit();
                    })
                    .unwrap();
            }
        }
    }

    fn run_event(
        &mut self,
        event: Event<()>,
        _control_flow: &EventLoopWindowTarget<()>,
        entities_meta: &mut NonSendMut<EntitiesMeta>,
        entity: &mut NonSendMut<EntityMeta>,
    ) {
        self.on_new_frame(&event);

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    _control_flow.exit();
                }
                WindowEvent::Resized(extent) => {
                    self.resize_event();
                }
                WindowEvent::RedrawRequested => {
                    self.on_draw(entities_meta, entity);
                }

                _ => {}
            },
            Event::DeviceEvent { device_id, ref event } => match event {
                event::DeviceEvent::MouseMotion { delta } => {
                    self.on_mouse_motion(delta);
                }
                event::DeviceEvent::Key(x) => match x.physical_key {
                    winit::keyboard::PhysicalKey::Code(keycode) => {
                        self.on_key_press(x, keycode);
                    }
                    winit::keyboard::PhysicalKey::Unidentified(_) => todo!(),
                },
                _ => {}
            },
            Event::AboutToWait => {}
            // happens after ever new event
            Event::NewEvents(_) => {}
            Event::LoopExiting => {
                // Cleanup resources here
                self.on_destroy();
            }

            _ => {}
        }
    }
}

impl ImguiApp {
    pub fn on_new(event_loop: &EventLoop<()>) -> Self {
        let vulkan = VulkanContext::new(event_loop, 2, true);

        Self {
            vulkan,
            last_frame: Instant::now(),
            resize: false,
            inspector: InspectorWindow::new(),
            hierachy: HierachyWindow::new(),
            exit: false,
        }
    }

    fn on_new_frame(&mut self, event: &winit::event::Event<()>) {
        let now = Instant::now();
        let delta_time = now.duration_since(self.last_frame);

        // TODO, make this into 1 update function, no reason to have them seperate
        self.vulkan.imgui.as_mut().unwrap().update_delta_time(delta_time);
        self.last_frame = now;
        self.vulkan.imgui.as_mut().unwrap().process_event_imgui(&self.vulkan.window, &event);
    }

    fn on_draw(&mut self, entities: &mut NonSendMut<EntitiesMeta>, entity: &mut NonSendMut<EntityMeta>) {
        self.vulkan.prepare_frame(&mut self.resize);

        if self.resize {
            unsafe {
                self.vulkan.device.device_wait_idle().unwrap();
                println!("Window extent before: {:?}", self.vulkan.window_extent);
                self.vulkan.recreate_swapchain();
                println!("Window extent now: {:?}", self.vulkan.window_extent);
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

        let entity_meta = self
            .hierachy
            .draw_hierachy(&mut ui, self.vulkan.window.scale_factor() as f32, self.vulkan.window_extent, entities);
        let entity_meta = self
            .inspector
            .draw_inspector(&mut ui, self.vulkan.window.scale_factor() as f32, self.vulkan.window_extent, entity_meta);

        if entity_meta.is_some() {
            let d = entity_meta.as_mut().unwrap();
            entity.deref_mut().id = d.id;
            entity.deref_mut().components.clear();

            for cmp in d.components.iter() {
                let deref = entity.deref_mut();
                deref.components.push(cmp.clone());
            }
        }

        imgui.render(
            self.vulkan.window_extent,
            &self.vulkan.swapchain.images[self.vulkan.swapchain.image_index as usize],
            self.vulkan.current_frame,
            &mut self.vulkan.resources.get_buffer_storage(),
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
