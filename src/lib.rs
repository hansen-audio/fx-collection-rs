// Copyright(c) 2021 Hansen Audio.

pub const NUM_CHANNELS: usize = 4;
pub type AudioFrame = [f32; NUM_CHANNELS];

pub mod cbindings;
pub mod stereo_delay;
pub mod trance_gate;
