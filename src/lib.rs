// Copyright(c) 2021 Hansen Audio.

pub type RealType = f32;
pub type AudioFrame = [RealType; 4];

pub mod cbindings;
mod detail;
pub mod trance_gate;
