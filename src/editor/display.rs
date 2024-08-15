use bevy::prelude::Vec3;
use imgui::internal;
const MAX_NAME_WIDTH: f32 = 100.0;
const PADDING: f32 = 10.0;

pub fn display_scalar<T>(ui: &mut imgui::Ui, label: &str, scalar: &mut T, space: f32)
where
    T: internal::DataTypeKind,
{
    let id = ui.push_id(label);
    ui.text(label);
    ui.same_line_with_pos(space);
    ui.input_scalar("##hidden", scalar).build();
    id.end();
}

pub fn display_scalar_2(ui: &mut imgui::Ui, field_name: &str, id: [&str; 2], scalar: &mut [f32; 2]) {
    let cursor_y = ui.cursor_pos()[1];
    ui.set_cursor_pos([5.0, cursor_y]);
    ui.text(field_name);

    let full_width = unsafe { imgui::sys::igGetWindowWidth() };

    let per_item_width = (full_width - MAX_NAME_WIDTH) / 3.0;

    let item_width = (full_width - MAX_NAME_WIDTH - PADDING * 4.0) / 3.0;
    for i in 0..3 {
        let id = ui.push_id(id[i]);

        ui.same_line_with_pos(MAX_NAME_WIDTH + per_item_width * i as f32);
        ui.text(SCALAR_ITEM_NAME[i]);
        ui.same_line_with_pos(MAX_NAME_WIDTH + PADDING + per_item_width * i as f32);
        ui.set_next_item_width(item_width);
        ui.input_scalar("##hidden", &mut scalar[i]).build();

        id.end();
    }
}

const SCALAR_ITEM_NAME: [&str; 3] = ["X", "Y", "Z"];

pub fn display_scalar_3(ui: &mut imgui::Ui, field_name: &str, id: [&str; 3], scalar: &mut Vec3) {
    let cursor_y = ui.cursor_pos()[1];
    ui.set_cursor_pos([5.0, cursor_y]);
    ui.text(field_name);

    let full_width = unsafe { imgui::sys::igGetWindowWidth() };

    let per_item_width = (full_width - MAX_NAME_WIDTH) / 3.0;

    let item_width = (full_width - MAX_NAME_WIDTH - PADDING * 4.0) / 3.0;
    for i in 0..3 {
        let id = ui.push_id(id[i]);

        ui.same_line_with_pos(MAX_NAME_WIDTH + per_item_width * i as f32);
        ui.text(SCALAR_ITEM_NAME[i]);
        ui.same_line_with_pos(MAX_NAME_WIDTH + PADDING + per_item_width * i as f32);
        ui.set_next_item_width(item_width);
        ui.input_scalar("##hidden", &mut scalar[i]).build();

        id.end();
    }
}

pub fn display_boolean(ui: &mut imgui::Ui, label: &str, scalar: &mut bool, space: f32) {
    let id = ui.push_id(label);
    ui.text(label);
    ui.same_line_with_pos(space);
    ui.checkbox("##hidden", scalar);
    id.end();
}

pub fn display_enum(ui: &mut imgui::Ui, label: &str, enum_types: &[&str], current_index: &mut i32, space: f32) {
    let id = ui.push_id(label);
    ui.text(label);
    ui.same_line_with_pos(space);
    ui.list_box(label, current_index, enum_types, enum_types.len() as i32);
    id.end();
}
