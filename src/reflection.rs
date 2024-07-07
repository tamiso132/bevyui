//! Illustrates how "reflection" works in Bevy.
//!
//! Reflection provides a way to dynamically interact with Rust types, such as accessing fields
//! by their string name. Reflection is a core part of Bevy and enables a number of interesting
//! features (like scenes).
use std::any::{Any, TypeId};

use bevy::{
    ecs::{component::ComponentId, observer::TriggerTargets},
    prelude::*,
    reflect::{
        serde::{ReflectDeserializer, ReflectSerializer},
        DynamicStruct,
    },
    scene::ron,
};
use bevy_reflect::{serde::TypedReflectDeserializer, GetTypeRegistration, TypeData, TypeRegistration, TypeRegistry, TypeRegistryArc};
use serde::de::DeserializeSeed;

/// Deriving `Reflect` implements the relevant reflection traits. In this case, it implements the
/// `Reflect` trait and the `Struct` trait `derive(Reflect)` assumes that all fields also implement
/// Reflect.
///
/// All fields in a reflected item will need to be `Reflect` as well. You can opt a field out of
/// reflection by using the `#[reflect(ignore)]` attribute.
/// If you choose to ignore a field, you need to let the automatically-derived `FromReflect` implementation
/// how to handle the field.
/// To do this, you can either define a `#[reflect(default = "...")]` attribute on the ignored field, or
/// opt-out of `FromReflect`'s auto-derive using the `#[reflect(from_reflect = false)]` attribute.
#[derive(Reflect, Default)]
#[reflect(Default)]
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

pub fn setup_reflection(world: &World) {
    let type_registry = world.get_resource::<AppTypeRegistry>().unwrap();
    let b = type_registry.write().add_registration(Bar::get_type_registration());
    println!("registered: {}", b);
}

pub fn setup_entities(mut commands: Commands, world: &World) {
    let cmd = commands.spawn(Bar::default());
    let entity = commands.spawn(Bar::default());
}

pub fn list_all_components(world: &World, type_registry: Res<AppTypeRegistry>) {
    let type_registry = type_registry.read();
    for entity in world.iter_entities() {
        println!("Entity: {:?}", entity.id());

        let archetype = entity.archetype();

        for component_id in archetype.components() {
            if let Some(component_info) = world.components().get_info(component_id) {
                println!("Component info, {}", component_info.name());

                let reflect_component = type_registry.get(component_info.type_id().unwrap()).unwrap();

                match reflect_component.type_info() {
                    bevy_reflect::TypeInfo::Struct(x) => {
                        for i in 0..x.field_len() {
                            let field_info = x.field_at(i).unwrap().type_path_table();

                            let ident = field_info.ident().unwrap();
                            let name = x.field_names().get(i).unwrap();

                            println!("Ident: {}", ident);
                            println!("name: {}", name);
                        }
                    }
                    bevy_reflect::TypeInfo::TupleStruct(x) => todo!(),
                    bevy_reflect::TypeInfo::Tuple(x) => todo!(),
                    bevy_reflect::TypeInfo::List(x) => todo!(),
                    bevy_reflect::TypeInfo::Array(x) => todo!(),
                    bevy_reflect::TypeInfo::Map(x) => todo!(),
                    bevy_reflect::TypeInfo::Enum(x) => todo!(),
                    bevy_reflect::TypeInfo::Value(x) => todo!(),
                }
                let component_type_name = component_info.name();
                println!("  Component: {}", component_type_name);
            }
        }
    }
    // let registration = type_registry.get(std::any::TypeId::of::<Bar>()).unwrap();
    // let deserialized = TypedReflectDeserializer::new(registration, &type_registry)
    //     .deserialize(&mut ron::Deserializer::from_str(input).unwrap())
    //     .unwrap();
}
