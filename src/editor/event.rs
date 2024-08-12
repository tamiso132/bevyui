use std::ops::Deref;

use bevy::{
    input::{
        mouse::{MouseButtonInput, MouseWheel},
        ButtonInput,
    },
    prelude::{DetectChanges, EventReader, KeyCode, Res},
    window::*,
};
