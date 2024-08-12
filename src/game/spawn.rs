use bevy::{
    asset::Handle,
    prelude::{default, Commands, Transform},
    sprite::SpriteBundle,
};
use bevy_rapier2d::prelude::{Collider, LockedAxes, RigidBody};

use crate::editor::ReflectionMarker;

use super::{Crate, Goal, Position, Size, OBJECT_SIZE};

pub fn spawn_crate(commands: &mut Commands, mut position: Position, texture_crate: Handle<bevy::prelude::Image>) {
    position.x *= OBJECT_SIZE as isize;
    position.y *= OBJECT_SIZE as isize;

    commands.spawn((
        position,
        Crate { goal_crate: false },
        RigidBody::Dynamic,
        SpriteBundle {
            texture: texture_crate,
            transform: Transform::from_xyz(position.x as f32, position.y as f32, 0.),
            ..default()
        },
        LockedAxes::ROTATION_LOCKED,
        Collider::cuboid(32.0 / 2.0, 32.0 / 2.0),
    ));
}

pub fn spawn_crate_goal(commands: &mut Commands, mut position: Position, texture_goal: Handle<bevy::prelude::Image>) {
    position.x *= OBJECT_SIZE as isize;
    position.y *= OBJECT_SIZE as isize;

    commands.spawn((
        SpriteBundle {
            texture: texture_goal,
            transform: Transform::from_xyz(position.x as f32, position.y as f32, 0.),
            ..default()
        },
        Crate { goal_crate: true },
        LockedAxes::ROTATION_LOCKED,
        Collider::cuboid(32.0 / 2.0, 32.0 / 2.0),
        RigidBody::Dynamic,
        position,
    ));
}

pub fn spawn_wall(commands: &mut Commands, mut position: Position, texture_wall: Handle<bevy::prelude::Image>) {
    position.x *= OBJECT_SIZE as isize;
    position.y *= OBJECT_SIZE as isize;

    commands.spawn((
        position,
        SpriteBundle {
            texture: texture_wall,
            transform: Transform::from_xyz(position.x as f32, position.y as f32, 0.),
            ..default()
        },
        RigidBody::Fixed,
        Collider::cuboid(32.0 / 2.0, 32.0 / 2.0),
        ReflectionMarker,
    ));
}

pub fn spawn_goal(commands: &mut Commands, mut position: Position, texture_goal: Handle<bevy::prelude::Image>) {
    position.x *= OBJECT_SIZE as isize;
    position.y *= OBJECT_SIZE as isize;

    commands.spawn((
        position,
        SpriteBundle {
            texture: texture_goal,
            transform: Transform::from_xyz(position.x as f32, position.y as f32, -1.),
            ..default()
        },
        Goal,
    ));
}
