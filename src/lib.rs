// Copyright(c) 2021 Hansen Audio.

pub type Real = f32;
pub type AudioFrame = [Real; 4];

pub mod cbindings;
mod detail;
pub mod trance_gate;
