// Copyright(c) 2021 Hansen Audio.

pub const NUM_CHANNELS: usize = 4;
pub type AudioFrame = [f32; NUM_CHANNELS];

const DEFAULT_TEMPO_BPM: f32 = 120.;
const DEFAULT_SAMPLE_RATE: f32 = 44100.;
const NUM_STEREO_CHANNELS: usize = 2;

pub mod cbindings;
pub mod stereo_delay;
pub mod trance_gate;
