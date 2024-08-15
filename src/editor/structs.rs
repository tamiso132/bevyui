use std::alloc::Layout;

use bevy::{ecs::component::ComponentId, prelude::ReflectComponent};
use bevy_reflect::{FromType, Reflect};

use super::reflection::Bar;

#[derive(Clone)]
pub struct TestComponent {
    pub name: String,
    pub id: ComponentId,
    pub data: Vec<u8>,
    pub reflect: ReflectTypeData,
    pub layout: Layout,
    pub is_removed: bool,
}

pub trait TReflect {
    fn display_imgui(data: &mut [u8], ui: *mut imgui::Ui);
}

#[derive(Clone)]
pub struct ReflectTypeData {
    display_imgui: fn(data: &mut Vec<u8>, imgui: *mut imgui::Ui),
}
impl ReflectTypeData {
    pub fn display_imgui(&self, data: &mut Vec<u8>, imgui: *mut imgui::Ui) {
        (self.display_imgui)(data, imgui);
    }
}

impl<T: TReflect> FromType<T> for ReflectTypeData {
    fn from_type() -> Self {
        Self {
            display_imgui: |data, imgui| {
                T::display_imgui(data, imgui);
            },
        }
    }
}
