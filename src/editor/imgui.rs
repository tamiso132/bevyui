use ash::vk::{self, Extent2D};
use bevy::{
    app::{Plugin, Startup, Update},
    prelude::{Commands, Entity, NonSendMut, Res},
};
use imgui::{Condition, Ui};
use lazy_static::lazy_static;
use once_cell::unsync::Lazy;
use reflection::{EntitiesMeta, EntityMeta, Foo};
use std::{
    cmp::{max, min},
    collections::HashMap,
    mem::{transmute, ManuallyDrop},
    ops::{Add, Deref, DerefMut},
    ptr::copy_nonoverlapping,
    str::FromStr,
    sync::Mutex,
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

use super::reflection::{self, FieldType};

pub fn ptr_to_u64(src: *const u8, len: usize) -> u64 {
    assert!(len <= 8);

    let mut buffer = [0u8; 8];

    unsafe {
        std::ptr::copy_nonoverlapping(src, buffer.as_mut_ptr(), len);
        transmute(buffer)
    }
}

pub fn ptr_to_i64(src: *const i8, len: usize) -> i64 {
    assert!(len <= 8);

    let mut buffer = [0i8; 8];

    unsafe {
        std::ptr::copy_nonoverlapping(src, buffer.as_mut_ptr(), len);
        transmute(buffer)
    }
}

pub fn ptr_to_string(src: *const u8) -> ManuallyDrop<String> {
    let mut s = unsafe { ManuallyDrop::new((src as *const String).read()) };
    s
}

// you can do this, becuase you know the aligment is always a power of two
/// Returns the aligned ptr and the amount it padded;
pub fn align_ptr(ptr: *mut u8, aligment: usize) -> *mut u8 {
    let ptr = ptr as usize;
    let aligned_ptr = (ptr + aligment - 1) & !(aligment - 1);

    assert!(aligned_ptr == ptr);

    aligned_ptr as *mut u8
}

pub fn align_ptr_with_padding_size(ptr: *mut u8, aligment: usize) -> usize {
    let ptr = ptr as usize;
    let aligned_ptr = (ptr + aligment - 1) & !(aligment - 1);
    aligned_ptr - ptr
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

    numbers: HashMap<(usize, usize), i32>,
    texts: HashMap<(usize, usize), String>,
    v: Vec<i32>,
}

impl InspectorWindow {
    fn new() -> Self {
        Self {
            window: true,
            width: 0.4,
            max_width: 300.0,
            active_entity: None,
            numbers: HashMap::new(),
            v: vec![50, 20],
            texts: HashMap::new(),
        }
    }
    fn draw_inspector(&mut self, ui: &mut Ui, hdpi_scale: f32, window_extent: Extent2D, entity: Option<EntityMeta>) -> &mut Option<EntityMeta> {
        let extent = window_extent;
        let inspector_width = window_extent.width as f32 * 0.5;

        let window_width = min(self.max_width as u32, inspector_width as u32) as f32;

        // Get a raw pointer to the original `Ui`
        let ui_ptr: *mut Ui = ui as *mut Ui;

        ui.window("inspector")
            .opened(&mut self.window)
            .position([extent.width as f32 - window_width, 0.0], Condition::Always)
            .size([window_width, extent.height as f32], Condition::Always)
            .build(|| {
                if entity.is_some() {
                    self.numbers.clear();
                    let mut entity_meta = entity.unwrap();

                    for i in 0..entity_meta.components.len() {
                        let component = &mut entity_meta.components[i];
                        component.reflect.display_imgui(&mut component.data, ui_ptr);
                        // component.reflect.display_imgui(&mut component.data, ui_ptr);
                        //
                        //    ui.text(component.name.clone());
                        // component.reflect.display_imgui(&mut component.data, ui);
                        // unsafe {
                        //     //let mut data_ptr = align_ptr(component.data.as_mut_ptr(), component.layout.align());

                        //     // for f in 0..component.fields.len() {
                        //     //     let field = &mut component.fields[f];
                        //     //     match field.type_ {
                        //     //         reflection::FieldType::Invalid => todo!(),
                        //     //         reflection::FieldType::USIZE
                        //     //         | reflection::FieldType::U64
                        //     //         | reflection::FieldType::U32
                        //     //         | reflection::FieldType::U16
                        //     //         | reflection::FieldType::U8 => {
                        //     //             // pad fields if needed
                        //     //             data_ptr = align_ptr(data_ptr, field.type_.get_aligment());

                        //     //             // add to hashmap
                        //     //             let number = ptr_to_u64(data_ptr, field.type_.get_size());
                        //     //             data_ptr = data_ptr.add(field.type_.get_size());
                        //     //             self.numbers.insert((i, f), number as i32);
                        //     //         }
                        //     //         reflection::FieldType::ISIZE
                        //     //         | reflection::FieldType::I64
                        //     //         | reflection::FieldType::I32
                        //     //         | reflection::FieldType::I16
                        //     //         | reflection::FieldType::I8 => {
                        //     //             // pad fields if needed
                        //     //             data_ptr = align_ptr(data_ptr, field.type_.get_aligment());

                        //     //             // add to hashmap
                        //     //             let number = ptr_to_i64(transmute(data_ptr), field.type_.get_size());
                        //     //             data_ptr = data_ptr.add(field.type_.get_size());
                        //     //             self.numbers.insert((i, f), number as i32);
                        //     //         }
                        //     //         reflection::FieldType::String => {
                        //     //             // pad fields if needed
                        //     //             data_ptr = align_ptr(data_ptr, field.type_.get_aligment());

                        //     //             // add to hashmap
                        //     //             let text = ptr_to_string(data_ptr);
                        //     //             data_ptr = data_ptr.add(field.type_.get_size());
                        //     //             self.texts.insert((i, f), text.to_string().clone());
                        //     //         }
                        //     //     }
                        //     // }
                        // }
                        //  ui.spacing();
                    }

                    self.active_entity = Some(entity_meta);
                } else if self.active_entity.is_some() {
                    let entity_meta = &mut self.active_entity.as_mut().unwrap();

                    let c = format!("Entity: {}", entity_meta.id.index());
                    ui.text(c.as_str());

                    for i in 0..entity_meta.components.len() {
                        let component = &mut entity_meta.components[i];

                        if imgui::CollapsingHeader::new(&component.name).build(ui) {
                            component.reflect.display_imgui(&mut component.data, ui_ptr);
                        }
                        // component.reflect.display_imgui(&mut component.data, ui_ptr);

                        // unsafe {
                        //     for f in 0..component.fields.len() {
                        //         let field = &mut component.fields[f];

                        //         //     match field.type_ {
                        //         //         reflection::FieldType::Invalid => todo!(),
                        //         //         reflection::FieldType::USIZE
                        //         //         | reflection::FieldType::U64
                        //         //         | reflection::FieldType::U32
                        //         //         | reflection::FieldType::U16
                        //         //         | reflection::FieldType::U8 => {
                        //         //             // Align the ptr
                        //         //             data_ptr = align_ptr(data_ptr, field.type_.get_aligment());

                        //         //             // Get hashmap value
                        //         //             let key = (i, f);
                        //         //             let mut val = self.numbers.get_mut(&key).unwrap();

                        //         //             // create imgui input
                        //         //             ui.text(field.name.to_string());
                        //         //             let identifier = ui.push_id(field.name.to_string());
                        //         //             ui.input_int("##", &mut val).read_only(false).always_overwrite(true).build();
                        //         //             identifier.end();

                        //         //             // Update the value
                        //         //             let mut v = val.clone() as u64;
                        //         //             let val_ptr = (&mut v as *mut u64) as *mut u8;
                        //         //             let field_size = field.type_.get_size();
                        //         //             copy_nonoverlapping(val_ptr, data_ptr, field_size);
                        //         //             let mut test: u64 = 1;
                        //         //             let mut test2: u64 = 1;
                        //         //             let mut v = vec![1, 5, 6];
                        //         //             //for arrays
                        //         //             // ui.input_scalar_n("hello", &mut v);

                        //         //             // for strings
                        //         //             data_ptr = data_ptr.add(field.type_.get_size());
                        //         //         }
                        //         //         reflection::FieldType::ISIZE
                        //         //         | reflection::FieldType::I64
                        //         //         | reflection::FieldType::I32
                        //         //         | reflection::FieldType::I16
                        //         //         | reflection::FieldType::I8 => {
                        //         //             // Align the ptr
                        //         //             data_ptr = align_ptr(data_ptr, field.type_.get_aligment());
                        //         //             let key = (i, f);
                        //         //             let mut value = self.numbers.get_mut(&key).unwrap();
                        //         //             ui.text(field.name.to_string());
                        //         //             // let text_width = ui.calc_text_size(&field.name);
                        //         //             //   ui.same_line_with_spacing(0.0, 50.0 - text_width[0]);
                        //         //             let identifier = ui.push_id(field.name.to_string());
                        //         //             ui.input_int("##", &mut value).read_only(false).always_overwrite(true).build();

                        //         //             // Pad if needed

                        //         //             identifier.end();
                        //         //             let mut val = [value.clone() as i64];
                        //         //             let field_size = field.type_.get_size();
                        //         //             let val_ptr = ((&mut val as *mut i64) as *mut u8).add(8 - field_size);
                        //         //             // Pad the val to get the last bytes

                        //         //             copy_nonoverlapping(val_ptr, data_ptr, field_size);

                        //         //             data_ptr = data_ptr.add(field.type_.get_size());
                        //         //         }
                        //         //         reflection::FieldType::String => {
                        //         //             // Align the ptr
                        //         //             data_ptr = align_ptr(data_ptr, field.type_.get_aligment());
                        //         //             let key = (i, f);
                        //         //             let mut value = self.texts.get_mut(&key).unwrap();
                        //         //             ui.text(field.name.to_string());
                        //         //             //  let mut string = ptr_to_string(data_ptr);
                        //         //             let mut ss = "ghggggggggggggggggggggggggggggggggggggg".to_string();
                        //         //             // let text_width = ui.calc_text_size(&field.name);
                        //         //             //   ui.same_line_with_spacing(0.0, 50.0 - text_width[0]);
                        //         //             //copy_nonoverlapping((&mut ss as *mut String).cast(), (&mut string as *mut ManuallyDrop<String>), 1);
                        //         //             let identifier = ui.push_id(field.name.to_string());
                        //         //             ui.input_text("##", &mut value).read_only(false).always_overwrite(true).build();

                        //         //             // Pad if needed
                        //         //             let next = ((value) as *mut String).cast::<u8>();
                        //         //             identifier.end();
                        //         //             copy_nonoverlapping(next, data_ptr, 24);

                        //         //             data_ptr = data_ptr.add(field.type_.get_size());
                        //         //         }
                        //         //     }
                        //     }
                        // }
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
        mut entity: &mut NonSendMut<EntityMeta>,
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

        let mut entity_meta = self
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
