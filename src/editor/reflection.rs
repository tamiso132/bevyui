use bevy::{
    ecs::{component::ComponentId, observer::TriggerTargets},
    prelude::*,
    ptr::Ptr,
};
use bevy_reflect::{DynamicTypePath, FromType, GetTypeRegistration, TypeData, TypeRegistration, Typed};
use imgui::{CollapsingHeader, Ui};
use std::{
    alloc::Layout,
    any::Any,
    borrow::BorrowMut,
    mem::{transmute, ManuallyDrop},
    ptr::copy_nonoverlapping,
};
use voxelengine_proc::ImGuiFields;

use super::{
    imgui::align_ptr,
    structs::{ReflectTypeData, TReflect, TestComponent},
    ReflectionMarker,
};

#[derive(Reflect, Default, Component, ImGuiFields)]
#[reflect(Component)]
pub struct Foo {
    a: usize,
}

impl TReflect for Foo {
    fn display_imgui(data: &mut Vec<u8>, imgui: *mut imgui::Ui) {
        unsafe {
            let ptr = align_ptr(data.as_mut_ptr(), align_of::<Foo>()).cast::<Foo>();
            let bar = &mut *ptr;
            let imgui = &mut *imgui;
            bar.render_imgui(imgui);
        }
    }
}

/// This `Bar` type is used in the `nested` field on the `Test` type. We must derive `Reflect` here
/// too (or ignore it)
#[derive(Component, Reflect, Debug, Default, ImGuiFields)]
#[reflect(Component)]
pub struct Bar {
    pub bba: String,
    pub b: u8,
    pub t: u64,
    pub l: u8,
}

impl TReflect for Bar {
    fn display_imgui(data: &mut Vec<u8>, imgui: *mut imgui::Ui) {
        unsafe {
            let ptr = align_ptr(data.as_mut_ptr(), align_of::<Bar>()).cast::<Bar>();
            let bar = &mut *ptr;
            let imgui = &mut *imgui;
            bar.render_imgui(imgui);
        }
    }
}

impl TReflect for Transform {
    fn display_imgui(data: &mut Vec<u8>, imgui: *mut imgui::Ui) {
        unsafe {
            let ptr = data.as_mut_ptr().cast::<Transform>();
            let bar = &mut *ptr;
            let imgui = &mut *imgui;

            let trans_ptr: *mut Vec3 = &mut bar.translation;
            let transform = trans_ptr.cast::<[f32; 3]>().as_mut().unwrap();

            let scale_ptr: *mut Vec3 = &mut bar.scale;
            let scale = scale_ptr.cast::<[f32; 3]>().as_mut().unwrap();

            imgui.input_float3("Transform", transform).build();
            imgui.input_float3("Scale", scale).build();
        }
    }
}

#[derive(Default, Reflect)]
#[reflect(Default)]
struct NonReflectedValue {
    _a: usize,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub enum FieldType {
    Invalid = 0,
    USIZE,
    U64,
    U32,
    U16,
    U8,
    ISIZE,
    I64,
    I32,
    I16,
    I8,
    String,
}

const SIZE_LOOK_UP_TABLE: [usize; 12] = [0, 8, 8, 4, 2, 1, 8, 8, 4, 2, 1, size_of::<String>()];
const ALIGMENT_LOOK_UP_TABLE: [usize; 12] = [0, 8, 8, 4, 2, 1, 8, 8, 4, 2, 1, align_of::<String>()];

impl FieldType {
    /// get the size of the field data in bytes
    pub fn get_size(&self) -> usize {
        unsafe {
            let index: u32 = transmute(*self);
            SIZE_LOOK_UP_TABLE[index as usize]
        }
    }
    pub fn get_aligment(&self) -> usize {
        ALIGMENT_LOOK_UP_TABLE[unsafe { transmute::<Self, u32>(*self) as usize }]
    }
}

impl Default for FieldType {
    fn default() -> Self {
        Self::Invalid
    }
}

impl From<&str> for FieldType {
    fn from(value: &str) -> Self {
        match value {
            "usize" => Self::USIZE,
            "u64" => Self::U64,
            "u32" => Self::U32,
            "u16" => Self::U16,
            "u8" => Self::U8,

            "isize" => Self::ISIZE,
            "i64" => Self::I64,
            "i32" => Self::I32,
            "i16" => Self::I16,
            "i8" => Self::I8,

            "String" => Self::String,

            _ => Self::Invalid,
        }
    }
}
#[repr(C)]
#[derive(Default, Clone)]
pub struct Field {
    pub type_: FieldType,
    pub name: String,
}

impl Field {
    pub fn new(name: String, type_: FieldType) -> Self {
        Field { name, type_ }
    }
}

#[repr(C)]
pub struct Component {
    pub name: String,
    pub layout: Layout,
    pub fields: Vec<Field>,
    pub id: ComponentId,
    pub data: Vec<u8>,
}

impl Component {
    pub fn clone(&self) -> Self {
        let mut fields: Vec<Field> = vec![];
        for i in 0..self.fields.len() {
            fields.push(self.fields[i].clone());
        }

        Self { name: self.name.clone(), layout: self.layout.clone(), fields, data: self.data.clone(), id: self.id }
    }
}
#[repr(C)]
pub struct EntityMeta {
    /// will be used in order to reflect changes later on
    pub id: Entity,
    // used in order to display the information
    pub components: Vec<TestComponent>,
}

impl EntityMeta {
    pub fn clone(&self) -> Self {
        let mut v = Vec::with_capacity(self.components.len());

        for i in 0..self.components.len() {
            v.push(self.components[i].clone());
        }

        Self { id: self.id, components: v }
    }
}

impl Default for EntityMeta {
    fn default() -> Self {
        Self { id: Entity::from_raw(0), components: vec![] }
    }
}

#[derive(Default)]
pub struct EntitiesMeta {
    pub data: Vec<EntityMeta>,
}

pub fn setup_reflection(world: &World) {
    let type_registry = world.get_resource::<AppTypeRegistry>().unwrap();
    let b = type_registry.write().add_registration(Bar::get_type_registration());
    let b = type_registry.write().add_registration(Foo::get_type_registration());

    type_registry.write().register_type_data::<Bar, ReflectTypeData>();
    type_registry.write().register_type_data::<Foo, ReflectTypeData>();
    type_registry.write().register_type_data::<Transform, ReflectTypeData>();
}

// pub fn setup_entities(mut commands: Commands, world: &World) {
//     let cmd = commands.spawn(Bar::default());
//     let entity = commands.spawn(Bar::default());
// }

pub fn overwrite_value(entities_meta: Vec<EntityMeta>) {
    for e in entities_meta {}
}

pub fn mutate_entities_data(enity_ref: NonSendMut<EntityMeta>) {}
// query: Query<(EntityRef, &ReflectionMarker)>, mut meta: ResMut<EntitiesMeta>

// if "bevy_sprite::bundle::SpriteBundle"
// if "bevy_rapier2d::dynamics::rigid_body::RigidBody"
// if "bevy_rapier2d::geometry::collider::Collider"
// if "bevy_rapier2d::dynamics::rigid_body::LockedAxes"

pub fn ignore(name: &str) -> bool {
    if name == "bevy_sprite::bundle::SpriteBundle"
        || name == "bevy_rapier2d::dynamics::rigid_body::RigidBody"
        || name == "bevy_rapier2d::geometry::collider::Collider"
        || name == "bevy_rapier2d::dynamics::rigid_body::LockedAxes"
        || name == "bevy_transform::components::global_transform::GlobalTransform"
        || name == "bevy_sprite::sprite::Sprite"
        || name == "bevy_asset::handle::Handle<bevy_render::texture::image::Image>"
        || name == "bevy_render::view::visibility::Visibility"
        || name == "bevy_render::view::visibility::ViewVisibility"
        || name == "bevy_render::view::visibility::InheritedVisibility"
        || name == "bevy_render::primitives::Aabb"
    {
        return true;
    }
    return false;
}

pub fn parse_world_entities_data(world: &mut World) {
    let mut entities_meta = vec![];
    {
        let mut query = world.query::<(EntityRef, Entity, &ReflectionMarker)>();

        let type_registry = world.get_resource::<AppTypeRegistry>().unwrap().0.read();
        for (entity, e, _) in query.iter(&world) {
            let archetype = entity.archetype();

            entities_meta.push(EntityMeta { id: entity.id(), components: vec![] });
            let last_index = entities_meta.len() - 1;
            let entity_meta = &mut entities_meta[last_index];
            for component_id in archetype.components() {
                if let Some(component_info) = world.components().get_info(component_id) {
                    if ignore(component_info.name()) {
                        continue;
                    } else {
                        if type_registry.get(component_info.type_id().unwrap()).is_none() {
                            continue;
                        }
                    }

                    let is_reflect = type_registry.get(component_info.type_id().unwrap());
                    match is_reflect {
                        Some(reflect_component) => {
                            let u = reflect_component.clone();
                            let reflect_trait = type_registry.get_type_data::<ReflectTypeData>(u.type_id()).unwrap().clone();
                            let data = entity.get_by_id(component_id).unwrap();
                            let layout = component_info.layout();

                            let mut d: Vec<u8> = vec![0; component_info.layout().size()];
                            unsafe {
                                copy_nonoverlapping(data.as_ptr(), d.as_mut_ptr(), component_info.layout().size());
                            }
                            entity_meta.components.push(TestComponent {
                                name: component_info.name().to_owned(),
                                id: component_id,
                                data: d,
                                reflect: reflect_trait,
                                layout,
                            });
                            // // if special_treatment(component_info.name()) {
                            // //     entity_meta.components.push(Component {
                            // //         name: reflect_component.type_info().type_path_table().short_path().to_owned(), // TODO, can do this
                            // //         fields: vec![],
                            // //         layout: component_info.layout(),
                            // //         data: vec![],
                            // //         id: component_id,
                            // //     });
                            // // }

                            // let tuple = { parse(reflect_component, data.as_ptr(), component_info.layout().align()) };

                            // let mut d = vec![0; tuple.1];

                            // unsafe {
                            //     copy_nonoverlapping(data.as_ptr(), d.as_mut_ptr(), tuple.1);

                            //     entity_meta.components.push(Component {
                            //         name: reflect_component.type_info().type_path_table().short_path().to_owned(), // TODO, can do this
                            //         fields: tuple.0,
                            //         layout: component_info.layout(),
                            //         data: d,
                            //         id: component_id,
                            //     });
                            // }
                        }
                        None => {}
                    }
                }
            }
        }
    }
    world.non_send_resource_mut::<EntitiesMeta>().as_mut().data = entities_meta;
}

pub fn mutate_data(world: &mut World) {
    let mut entity_meta;
    {
        entity_meta = world.non_send_resource_mut::<EntityMeta>().clone();
    }

    let entity_ref = world.get_entity(entity_meta.id).unwrap();

    for component in &mut entity_meta.components {
        let data_ptr = entity_ref.get_by_id(component.id).unwrap();
        let ptr = align_ptr(data_ptr.as_ptr(), component.layout.align());
        unsafe {
            std::ptr::copy_nonoverlapping(component.data.as_mut_ptr(), ptr, component.layout.size());
        }
    }
}

pub fn write_out_data(query: Query<&Bar>) {
    for bar in query.iter() {
        println!("bar: {}", bar.bba);
    }
}

fn parse(generic_component: &TypeRegistration, data_ptr: *mut u8, alignment: usize) -> (Vec<Field>, usize) {
    let mut new_ptr = align_ptr(data_ptr, alignment);

    match generic_component.type_info() {
        bevy_reflect::TypeInfo::Struct(x) => {
            let mut fields = vec![];
            for i in 0..x.field_len() {
                let field = x.field_at(i).unwrap().type_path_table();
                let field_name = x.field_at(i).unwrap().name().to_owned();
                let field_ident = field.ident().unwrap();
                let field_type = FieldType::from(field_ident);

                // add offset of field
                new_ptr = align_ptr(new_ptr, field_type.get_aligment());
                unsafe {
                    new_ptr = new_ptr.add(field_type.get_size());
                }
                fields.push(Field::new(field_name, field_type));
            }
            (fields, new_ptr as usize - data_ptr as usize)
        }
        bevy_reflect::TypeInfo::TupleStruct(_) => todo!(),
        bevy_reflect::TypeInfo::Tuple(_) => todo!(),
        bevy_reflect::TypeInfo::List(_) => todo!(),
        bevy_reflect::TypeInfo::Array(_) => todo!(),
        bevy_reflect::TypeInfo::Map(_) => todo!(),
        bevy_reflect::TypeInfo::Enum(_) => todo!(),
        bevy_reflect::TypeInfo::Value(_) => todo!(),
    }
}
