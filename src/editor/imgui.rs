use std::{
    cmp::{max, min},
    collections::HashMap,
    mem::transmute,
    ptr::copy_nonoverlapping,
    time::{Duration, Instant},
};

use ash::vk::{self, Extent2D};
use bevy::{
    app::{Plugin, Startup, Update},
    prelude::{Commands, Entity, NonSendMut, Res},
};
use imgui::{Condition, Ui};
use reflection::{EntitiesMeta, EntityMeta, Foo, ReflectionMarker};
use voxelengine::{
    vulkan::{util, VulkanContext},
    App::ApplicationTrait,
};
use winit::{
    event::{self, Event, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
    platform::{pump_events::EventLoopExtPumpEvents, run_on_demand::EventLoopExtRunOnDemand},
};

use super::reflection::{self, FieldType};

pub fn ptr_to_u64(src: *const u8, len: usize) -> u64 {
    assert!(len <= 8);

    let mut buffer = [0u8; 8];

    unsafe {
        std::ptr::copy_nonoverlapping(src, buffer.as_mut_ptr(), len);
        transmute(buffer)
    }
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

    fn draw_hierachy<'a>(&mut self, ui: &mut Ui, window_extent: Extent2D, entities: &'a mut NonSendMut<EntitiesMeta>) -> Option<&'a mut EntityMeta> {
        let extent = window_extent;
        let inspector_width = window_extent.width as f32 * 0.3;

        let mut entity_pressed = None;
        let window_width = min(self.max_width as u32, inspector_width as u32) as f32;

        let w = ui
            .window("Hierachy")
            .opened(&mut self.window)
            .position([0.0, 0.0], Condition::Always)
            .size([window_width, extent.height as f32], Condition::Always);

        w.build(|| {
            let c_text = "this is a component";
            let mut index: i32 = -1;
            for i in 0..entities.data.len() {
                let entity = &mut entities.data[i];
                let s = format!("{}", entity.id.index());
                if ui.button(s) {
                    index = i as i32;
                }
                ui.spacing();
            }
            if index > 0 {
                entity_pressed = Some(&mut entities.data[index as usize]);
            }
        });
        entity_pressed
    }
}

#[derive(Default)]
struct InspectorWindow {
    headers: Vec<bool>,
    window: bool,
    width: f32,
    max_width: f32,

    active_entity: Option<EntityMeta>,

    numbers: HashMap<(usize, usize), i32>,
}

impl InspectorWindow {
    fn new() -> Self {
        Self {
            headers: vec![false, true, true, false],
            window: true,
            width: 0.4,
            max_width: 300.0,
            active_entity: None,
            numbers: HashMap::new(),
        }
    }
    fn draw_entity(&mut self, entity: &mut EntityMeta) {}
    fn draw_inspector(&mut self, ui: &mut Ui, window_extent: Extent2D, entity: Option<&mut EntityMeta>) {
        let extent = window_extent;
        let inspector_width = window_extent.width as f32 * 0.3;

        let window_width = min(self.max_width as u32, inspector_width as u32) as f32;

        let w = ui
            .window("Inspector")
            .opened(&mut self.window)
            .position([extent.width as f32 - window_width, 0.0], Condition::Always)
            .size([window_width, extent.height as f32], Condition::Always);

        w.build(|| {
            let c_text = "this is a component";

            if entity.is_some() {
                self.numbers.clear();

                let entity_meta = entity.unwrap();
                let components_max = entity_meta.components.len();
                if components_max > self.headers.len() {
                    for _ in 0..components_max - self.headers.len() {
                        self.headers.push(false);
                    }
                }

                for i in 0..entity_meta.components.len() {
                    let component = &mut entity_meta.components[i];
                    ui.text(component.name.clone());
                    unsafe {
                        let align_offset = component.data.align_offset(component.layout.align());
                        let mut data_ptr = component.data.add(align_offset);
                        let u = [150];
                        copy_nonoverlapping(u.as_ptr(), component.data, 1);

                        for f in 0..component.fields.len() {
                            let field = &mut component.fields[f];
                            match field.type_ {
                                reflection::FieldType::Invalid => todo!(),
                                reflection::FieldType::USIZE
                                | reflection::FieldType::U64
                                | reflection::FieldType::U32
                                | reflection::FieldType::U16
                                | reflection::FieldType::U8 => {
                                    let number = ptr_to_u64(data_ptr, field.type_.get_size());
                                    data_ptr = data_ptr.add(field.type_.get_size());
                                    self.numbers.insert((i, f), number as i32);
                                    let value = self.numbers.get_mut(&(i, f)).unwrap();
                                    ui.input_int(field.name.clone(), value).build();
                                }
                                reflection::FieldType::ISIZE => todo!(),
                                reflection::FieldType::I64 => todo!(),
                                reflection::FieldType::I32 => todo!(),
                                reflection::FieldType::I16 => todo!(),
                                reflection::FieldType::I8 => todo!(),
                                reflection::FieldType::String => todo!(),
                            }
                        }
                    }
                    ui.spacing();
                }

                self.active_entity = Some(entity_meta.clone());
            } else if self.active_entity.is_some() {
                let entity_meta = &mut self.active_entity.as_mut().unwrap();
                let components_max = entity_meta.components.len();
                if components_max > self.headers.len() {
                    for _ in 0..components_max - self.headers.len() {
                        self.headers.push(false);
                    }
                }

                for i in 0..entity_meta.components.len() {
                    let component = &mut entity_meta.components[i];
                    ui.text(component.name.clone());
                    unsafe {
                        for f in 0..component.fields.len() {
                            let field = &mut component.fields[f];
                            match field.type_ {
                                reflection::FieldType::Invalid => todo!(),
                                reflection::FieldType::USIZE
                                | reflection::FieldType::U64
                                | reflection::FieldType::U32
                                | reflection::FieldType::U16
                                | reflection::FieldType::U8 => {
                                    let key = (i, f);
                                    let mut value = self.numbers.get_mut(&key).unwrap();
                                    ui.text(field.name.to_string());
                                    // let text_width = ui.calc_text_size(&field.name);
                                    //   ui.same_line_with_spacing(0.0, 50.0 - text_width[0]);
                                    ui.input_int("u", &mut value).build();
                                }
                                reflection::FieldType::ISIZE => todo!(),
                                reflection::FieldType::I64 => todo!(),
                                reflection::FieldType::I32 => todo!(),
                                reflection::FieldType::I16 => todo!(),
                                reflection::FieldType::I8 => todo!(),
                                reflection::FieldType::String => todo!(),
                            }
                        }
                    }
                    ui.spacing();
                }
            }
        });
    }
}

pub struct ImguiApp {
    vulkan: VulkanContext,
    resize: bool,
    last_frame: Instant,

    pub entities: Vec<EntityMeta>,
    inspector: InspectorWindow,
    hierachy: HierachyWindow,

    exit: bool,
}

const HZ_MAX: i64 = (1000.0 / 60.0) as i64;

impl ImguiApp {
    pub fn run_non_block(&mut self, event_loop: &mut EventLoop<()>, mut entities_meta: &mut NonSendMut<EntitiesMeta>) {
        if !self.exit {
            event_loop.pump_events(Some(Duration::ZERO), |event, _control_flow| {
                self.run_event(event, _control_flow, &mut entities_meta);
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

    fn run_event(&mut self, event: Event<()>, _control_flow: &EventLoopWindowTarget<()>, entities_meta: &mut NonSendMut<EntitiesMeta>) {
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
                    self.on_draw(entities_meta);
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
            entities: vec![],
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

    fn on_draw(&mut self, entities: &mut NonSendMut<EntitiesMeta>) {
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

        // Self::draw_inspector(&mut ui, &mut self.window, self.vulkan.window_extent);
        let entity_meta = self.hierachy.draw_hierachy(&mut ui, self.vulkan.window_extent, entities);
        self.inspector.draw_inspector(&mut ui, self.vulkan.window_extent, entity_meta);

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
