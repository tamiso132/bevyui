use std::{alloc::Layout, any::Any, borrow::BorrowMut};

use bevy::{a11y::accesskit::Invalid, log::tracing_subscriber::field, prelude::*};
use bevy_reflect::{GetTypeRegistration, TypeData, TypeRegistration};

#[derive(Reflect, Default, Component)]
#[reflect(Component)]
pub struct Foo {
    a: usize,
    nested: Bar,
    #[reflect(ignore)]
    _ignored: NonReflectedValue,
}

/// This `Bar` type is used in the `nested` field on the `Test` type. We must derive `Reflect` here
/// too (or ignore it)
#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
pub struct Bar {
    b: usize,
    t: u32,
    bba: u16,
}

#[derive(Default, Reflect)]
#[reflect(Default)]
struct NonReflectedValue {
    _a: usize,
}
#[derive(Clone, Copy)]
enum FieldType {
    Invalid,
    UInt,
    IInt,
    Str,
}

impl From<&str> for FieldType {
    fn from(value: &str) -> Self {
        match value {
            "u32" | "u16" | "u8" => FieldType::UInt,
            "i32" | "i16" | "iu8" => FieldType::IInt,
            "Str" | "String" => FieldType::Str,
            _ => FieldType::Invalid,
        }
    }
}
struct Field {
    /// the reason you cannot have a ptr toward the field, is because it can change.
    pub data: Vec<u8>,
    pub name: String,
    pub type_: FieldType,
}

impl Field {
    pub fn new(name: String, data: Vec<u8>, type_: FieldType) -> Self {
        Field { name, data, type_ }
    }
}
pub struct Component {
    pub name: String,
    pub layout: Layout,
    pub fields: Vec<Field>,
}
pub struct EntityMeta {
    /// will be used in order to reflect changes later on
    pub id: Entity,
    // used in order to display the information
    pub components: Vec<Component>,
}

impl Default for EntityMeta {
    fn default() -> Self {
        Self { id: Entity::from_raw(0), components: vec![] }
    }
}

#[derive(Resource, Default)]
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
                            entity_meta.components.push(Component {
                                name: component_info.name().to_owned(),
                                fields: parse(reflect_component),
                                layout: component_info.layout(),
                            });
                        }
                        None => {}
                    }

                    let component_type_name = component_info.name();
                    println!("  Component: {}", component_type_name);
                }
            }
        }
    }
    world.resource_mut::<EntitiesMeta>().as_mut().data = entities_meta;
}

fn parse(generic_component: &TypeRegistration) -> Vec<Field> {
    match generic_component.type_info() {
        bevy_reflect::TypeInfo::Struct(x) => {
            let mut fields = vec![];

            for i in 0..x.field_len() {
                let field = x.field_at(i).unwrap().type_path_table();
                let field_name = x.field_at(i).unwrap().name().to_owned();
                let field_ident = field.ident().unwrap();
                fields.push(Field::new(field_name, vec![], FieldType::from(field_ident)));
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
