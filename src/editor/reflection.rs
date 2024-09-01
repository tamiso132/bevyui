use bevy::{
    ecs::{component::ComponentId, observer::TriggerTargets},
    prelude::*,
    ptr::Ptr,
};
use bevy_rapier2d::prelude::LockedAxes;
use bevy_reflect::{DynamicTypePath, FromType, GetTypeRegistration, TypeData, TypeRegistration, Typed};
use core::panic;
use imgui::{CollapsingHeader, Ui};
use std::{
    alloc::Layout,
    any::Any,
    borrow::BorrowMut,
    mem::{transmute, ManuallyDrop},
    ptr::copy_nonoverlapping,
};
use voxelengine_proc::{BevyField, ImGuiFields};

use super::{
    imgui::align_ptr,
    structs::{ReflectTypeData, TReflect, TestComponent},
    ReflectionMarker,
};

#[derive(Reflect, Default, Component, BevyField)]
#[reflect(Component)]
pub struct Foo {
    a: usize,
}


#[derive(Reflect, Default, Component, BevyField)]
#[reflect(Component)]
struct NumberWizard {
    max: i32,
    min: i32,
    guess_text: String,
    guess: i32,
}


// impl TReflect for Foo {
//     fn display_imgui(data: &mut Vec<u8>, imgui: *mut imgui::Ui) {
//         unsafe {
//             let ptr = align_ptr(data.as_mut_ptr(), align_of::<Foo>()).cast::<Foo>();
//             let bar = &mut *ptr;
//             let imgui = &mut *imgui;
//             bar.render_imgui(imgui);
//         }
//     }
// }

/// This `Bar` type is used in the `nested` field on the `Test` type. We must derive `Reflect` here
/// too (or ignore it)
#[derive(Component, Reflect, Debug, Default, BevyField)]
#[reflect(Component)]
pub struct Bar {
    pub bba: String,
    pub b: u8,
    pub t: u64,
    pub l: u8,
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
        let mut delete_components = Vec::with_capacity(self.components.len());
        for i in 0..self.components.len() {
            v.push(self.components[i].clone());
            delete_components.push(true);
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

    Bar::register(type_registry);
    Foo::register(type_registry);

    type_registry.write().register_type_data::<Transform, ReflectTypeData>();
}

// pub fn ignore(name: &str) -> bool {
//     if name == "bevy_rapier2d::dynamics::rigid_body::RigidBody"
//         || name == "bevy_rapier2d::geometry::collider::Collider"
//         || name == "bevy_transform::components::global_transform::GlobalTransform"
//         || name == "bevy_sprite::sprite::Sprite"
//         || name == "bevy_asset::handle::Handle<bevy_render::texture::image::Image>"
//         || name == "bevy_render::view::visibility::InheritedVisibility"
//         || name == "bevy_render::primitives::Aabb"
//     {
//         return true;
//     }
//     return false;
// }

pub fn parse_world_entities_data(world: &mut World) {
    let mut entities_meta = vec![];
    {
        let mut query = world.query::<(EntityRef, Entity, &ReflectionMarker)>();

        let id_v = world.component_id::<LockedAxes>().unwrap();

        let type_registry = world.get_resource::<AppTypeRegistry>().unwrap().0.read();
        for (entity, e, _) in query.iter(&world) {
            let archetype = entity.archetype();
            entities_meta.push(EntityMeta { id: entity.id(), components: vec![] });
            let last_index = entities_meta.len() - 1;
            let entity_meta = &mut entities_meta[last_index];
            for component_id in archetype.components() {
                if let Some(component_info) = world.components().get_info(component_id) {
                    if id_v == component_info.id() {
                        panic!("LOCKED");
                        // let ptr = unsafe { world.get_by_id(entity.id(), id_v).unwrap().assert_unique() };
                        // let visibility = unsafe { ptr.deref_mut::<Visibility>() };
                        // *visibility = Visibility::Hidden;
                    }

                    let is_reflect = type_registry.get(component_info.type_id().unwrap());
                    match is_reflect {
                        Some(reflect_component) => match type_registry.get_type_data::<ReflectTypeData>(component_info.type_id().unwrap()) {
                            Some(reflect_trait) => {
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
                                    reflect: reflect_trait.clone(),
                                    layout,
                                    is_removed: false,
                                });
                            }
                            None => {}
                        },
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

    let v_id = world.component_id::<Visibility>().unwrap();

    entity_meta.id;

    for component in &mut entity_meta.components {
        match entity_ref.get_by_id(component.id) {
            Some(data_ptr) => {
                let ptr = unsafe { data_ptr.assert_unique() };
                // let ptr = align_ptr(data_ptr.as_ptr(), component.layout.align());
                let u = unsafe { *ptr.as_ptr().cast::<u8>() };
                if u == 2 {
                    println!("wtf?");
                }
                unsafe {
                    std::ptr::copy_nonoverlapping(component.data.as_mut_ptr(), ptr.as_ptr(), component.layout.size());
                }
            }
            None => {}
        }
    }
}

pub fn check_component_delete(mut cmd: Commands, mut entity_meta: NonSendMut<EntityMeta>) {
    let mut remove_index = -1;
    cmd.entity(entity_meta.id).log_components();
    for i in 0..entity_meta.components.len() {
        if entity_meta.components[i].is_removed {
            remove_index = i as i32;
            break;
        }
    }

    if remove_index >= 0 {
        let component = entity_meta.components.remove(remove_index as usize);
        cmd.entity(entity_meta.id).remove_by_id(component.id);
    }
}
