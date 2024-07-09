use std::{alloc::Layout, any::Any, borrow::BorrowMut, mem::transmute};

use bevy::{a11y::accesskit::Invalid, log::tracing_subscriber::field, prelude::*};
use bevy_reflect::{DynamicTypePath, GetTypeRegistration, TypeData, TypeRegistration};

#[derive(Reflect, Default, Component)]
#[reflect(Component)]
pub struct Foo {
    a: usize,
}

/// This `Bar` type is used in the `nested` field on the `Test` type. We must derive `Reflect` here
/// too (or ignore it)
#[repr(C)]
#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
pub struct Bar {
    pub b: usize,
    pub t: u32,
    pub bba: u16,
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

const SIZE_LOOK_UP_TABLE: [usize; 12] = [0, 8, 8, 4, 2, 1, 8, 8, 4, 2, 1, 24];

impl FieldType {
    /// get the size of the field data in bytes
    pub fn get_size(&self) -> usize {
        unsafe {
            let index: u32 = transmute(*self);
            SIZE_LOOK_UP_TABLE[index as usize]
        }
    }
}

impl Default for FieldType {
    fn default() -> Self {
        Self::Invalid
    }
}

impl From<&str> for (FieldType) {
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
    pub data: *mut u8,
}

impl Component {
    pub fn clone(&self) -> Self {
        let mut fields: Vec<Field> = vec![];
        for i in 0..self.fields.len() {
            fields.push(self.fields[i].clone());
        }

        Self { name: self.name.clone(), layout: self.layout, fields, data: self.data.clone() }
    }
}
#[repr(C)]
pub struct EntityMeta {
    /// will be used in order to reflect changes later on
    pub id: Entity,
    // used in order to display the information
    pub components: Vec<Component>,
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

#[derive(Component, Default)]
pub struct ReflectionMarker;

pub fn setup_reflection(world: &World) {
    let type_registry = world.get_resource::<AppTypeRegistry>().unwrap();
    let b = type_registry.write().add_registration(Bar::get_type_registration());
    let b = type_registry.write().add_registration(Foo::get_type_registration());
    println!("registered: {}", b);
}

// pub fn setup_entities(mut commands: Commands, world: &World) {
//     let cmd = commands.spawn(Bar::default());
//     let entity = commands.spawn(Bar::default());
// }

pub fn overwrite_value(entities_meta: Vec<EntityMeta>) {
    for e in entities_meta {}
}

pub fn mutate_entities_data(query: Query<(Entity, &ReflectionMarker)>) {
    for (e, _) in query.iter() {}
}
// query: Query<(EntityRef, &ReflectionMarker)>, mut meta: ResMut<EntitiesMeta>
pub fn parse_world_entities_data(world: &mut World) {
    let mut entities_meta = vec![];

    {
        let mut query = world.query::<(EntityRef, &ReflectionMarker)>();

        let type_registry = world.get_resource::<AppTypeRegistry>().unwrap().0.read();

        for (entity, _) in query.iter(&world) {
            let archetype = entity.archetype();

            entities_meta.push(EntityMeta { id: entity.id(), components: vec![] });
            let last_index = entities_meta.len() - 1;
            let entity_meta = &mut entities_meta[last_index];

            for component_id in archetype.components() {
                if let Some(component_info) = world.components().get_info(component_id) {
                    let is_reflect = type_registry.get(component_info.type_id().unwrap());
                    match is_reflect {
                        Some(reflect_component) => {
                            component_info.layout();
                            let data = entity.get_by_id(component_id).unwrap();
                            let u = [50];
                            unsafe {
                                std::ptr::copy(u.as_ptr(), data.as_ptr(), 1);
                                entity_meta.components.push(Component {
                                    name: reflect_component.type_info().type_path_table().short_path().to_owned(), // TODO, can do this
                                    fields: parse(reflect_component),
                                    layout: component_info.layout(),
                                    data: data.as_ptr(),
                                });
                            }
                        }
                        None => {}
                    }

                    let component_type_name = component_info.name();
                    println!("  Component: {}", component_type_name);
                }
            }
        }
    }
    world.non_send_resource_mut::<EntitiesMeta>().as_mut().data = entities_meta;
}

pub fn write_out_data(query: Query<&Bar>) {
    for bar in query.iter() {
        println!("b: {}", bar.b);
        println!("bba: {}", bar.bba);
        println!("t: {}", bar.t);
    }
}

fn parse(generic_component: &TypeRegistration) -> Vec<Field> {
    match generic_component.type_info() {
        bevy_reflect::TypeInfo::Struct(x) => {
            let mut fields = vec![];

            for i in 0..x.field_len() {
                let field = x.field_at(i).unwrap().type_path_table();
                let field_name = x.field_at(i).unwrap().name().to_owned();
                let field_ident = field.ident().unwrap();
                fields.push(Field::new(field_name, FieldType::from(field_ident)));
            }
            fields
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
