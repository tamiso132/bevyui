use bevy::{math::Vec3, prelude::*};
use bevy_rapier2d::prelude::{Collider, LockedAxes, RigidBody};

use super::{
    display,
    structs::{ReflectTypeData, TReflect},
};

pub fn register_bevy_types(registry: &AppTypeRegistry) {
    registry.write().register::<LockedAxes>();
    registry.write().register::<ViewVisibility>();
    registry.write().register::<RigidBody>();

    registry.write().register_type_data::<Transform, ReflectTypeData>();
    registry.write().register_type_data::<LockedAxes, ReflectTypeData>();
    registry.write().register_type_data::<ViewVisibility, ReflectTypeData>();
}

pub unsafe fn get_component<T>(data: &mut [u8], ui: *mut imgui::Ui) -> (&mut imgui::Ui, &mut T) {
    let ptr = data.as_mut_ptr().cast::<T>();
    let value = &mut *ptr;
    let ui = &mut *ui;

    (ui, value)
}

impl TReflect for Transform {
    fn display_imgui(data: &mut [u8], ui: *mut imgui::Ui) {
        unsafe {
            let ptr = data.as_mut_ptr().cast::<Transform>();
            let bar = &mut *ptr;
            let ui = &mut *ui;

            display::display_scalar_3(
                ui,
                "Translation",
                ["Transform::Translation::0", "Transform::Translation::1", "Transform::Translation::2"],
                &mut bar.translation,
            );
            let pos = ui.cursor_pos();
            ui.set_cursor_pos([0.0, pos[1] + 5.0]);
            display::display_scalar_3(ui, "Scale", ["Transform::Scale::0", "Transform::Scale::1", "Transform::Scale::2"], &mut bar.scale);
        }
    }
}

impl TReflect for GlobalTransform {
    fn display_imgui(data: &mut [u8], ui: *mut imgui::Ui) {
        unsafe {
            let ptr = data.as_mut_ptr().cast::<GlobalTransform>();
            let value = &mut *ptr;
            let ui = &mut *ui;
        }
    }
}

// /// Flag indicating that the [`RigidBody`] cannot translate along the `X` axis.
// const TRANSLATION_LOCKED_X = 1 << 0;
// /// Flag indicating that the [`RigidBody`] cannot translate along the `Y` axis.
// const TRANSLATION_LOCKED_Y = 1 << 1;
// /// Flag indicating that the [`RigidBody`] cannot translate along the `Z` axis.
// const TRANSLATION_LOCKED_Z = 1 << 2;
// /// Flag indicating that the [`RigidBody`] cannot translate along any direction.
// const TRANSLATION_LOCKED = Self::TRANSLATION_LOCKED_X.bits() | Self::TRANSLATION_LOCKED_Y.bits() | Self::TRANSLATION_LOCKED_Z.bits();
// /// Flag indicating that the [`RigidBody`] cannot rotate along the `X` axis.
// const ROTATION_LOCKED_X = 1 << 3;
// /// Flag indicating that the [`RigidBody`] cannot rotate along the `Y` axis.
// const ROTATION_LOCKED_Y = 1 << 4;
// /// Flag indicating that the [`RigidBody`] cannot rotate along the `Z` axis.
// const ROTATION_LOCKED_Z = 1 << 5;
// /// Combination of flags indicating that the [`RigidBody`] cannot rotate along any axis.
// const ROTATION_LOCKED = Self::ROTATION_LOCKED_X.bits() | Self::ROTATION_LOCKED_Y.bits() | Self::ROTATION_LOCKED_Z.bits();

fn convert_to_index(axes: &LockedAxes) -> u8 {
    match *axes {
        LockedAxes::TRANSLATION_LOCKED_X => 0,
        LockedAxes::TRANSLATION_LOCKED_Y => 1,
        LockedAxes::TRANSLATION_LOCKED_Z => 2,
        LockedAxes::TRANSLATION_LOCKED => 3,
        LockedAxes::ROTATION_LOCKED_X => 4,
        LockedAxes::ROTATION_LOCKED_Y => 5,
        LockedAxes::ROTATION_LOCKED_Z => 6,
        LockedAxes::ROTATION_LOCKED => 7,
        _ => {
            panic!()
        }
    }
}

fn convert_to_locked(index: u8) -> LockedAxes {
    match index {
        0 => LockedAxes::TRANSLATION_LOCKED_X,
        1 => LockedAxes::TRANSLATION_LOCKED_Y,
        2 => LockedAxes::TRANSLATION_LOCKED_Z,
        3 => LockedAxes::TRANSLATION_LOCKED,
        4 => LockedAxes::ROTATION_LOCKED_X,
        5 => LockedAxes::ROTATION_LOCKED_Y,
        6 => LockedAxes::ROTATION_LOCKED_Z,
        7 => LockedAxes::ROTATION_LOCKED,
        _ => {
            panic!()
        }
    }
}

const LOCKED_AXES_STR: [&str; 8] = [
    "TRANSLATION_LOCKED_X",
    "TRANSLATION_LOCKED_Y",
    "TRANSLATION_LOCKED_Z",
    "TRANSLATION_LOCKED",
    "ROTATION_LOCKED_X",
    "ROTATION_LOCKED_Y",
    "ROTATION_LOCKED_Z",
    "ROTATION_LOCKED",
];

impl TReflect for LockedAxes {
    fn display_imgui(data: &mut [u8], ui: *mut imgui::Ui) {
        unsafe {
            let ptr = data.as_mut_ptr().cast::<LockedAxes>();
            let value = &mut *ptr;
            let ui = &mut *ui;

            let mut list_box_index = convert_to_index(value) as i32;

            display::display_enum(ui, "LockedAxes", &LOCKED_AXES_STR, &mut list_box_index, 50.0);

            *value = convert_to_locked(list_box_index as u8);
        }
    }
}

impl TReflect for bevy_render::view::visibility::ViewVisibility {
    fn display_imgui(data: &mut [u8], ui: *mut imgui::Ui) {
        let (ui, value) = unsafe { get_component::<bool>(data, ui) };

        display::display_boolean(ui, "Render", value, 50.0);
    }
}
