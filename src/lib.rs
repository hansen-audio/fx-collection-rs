// Copyright(c) 2021 Hansen Audio.

pub type Real = f32;
pub const NUM_CHANNELS: usize = 2;
pub type AudioFrame = [Real; NUM_CHANNELS];

pub mod cbindings;
mod detail;
pub mod trance_gate;
