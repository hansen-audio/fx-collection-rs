// Copyright(c) 2021 Hansen Audio.

use crate::{trance_gate, AudioFrame};

//-----------------------------------------------------------------------------
// https://firefox-source-docs.mozilla.org/writing-rust-code/ffi.html
#[no_mangle]
pub unsafe extern "C" fn create_trance_gate() -> *mut trance_gate::TranceGate {
    let trance_gate = trance_gate::TranceGate::new();
    Box::into_raw(Box::new(trance_gate))
}

#[no_mangle]
pub unsafe extern "C" fn destroy(trance_gate: *mut trance_gate::TranceGate) {
    drop(Box::from_raw(trance_gate));
}

//-----------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn set_tempo(trance_gate: &mut trance_gate::TranceGate, tempo_bpm: f32) {
    trance_gate.set_tempo(tempo_bpm);
}

#[no_mangle]
pub unsafe extern "C" fn trigger(
    trance_gate: &mut trance_gate::TranceGate,
    delay_len: f32,
    fade_in_len: f32,
) {
    trance_gate.trigger(delay_len, fade_in_len);
}

#[no_mangle]
pub unsafe extern "C" fn reset(trance_gate: &mut trance_gate::TranceGate) {
    trance_gate.reset();
}

#[no_mangle]
pub unsafe extern "C" fn reset_step_pos(trance_gate: &mut trance_gate::TranceGate, value: usize) {
    trance_gate.reset_step_pos(value);
}

#[no_mangle]
pub unsafe extern "C" fn get_step_pos(trance_gate: &mut trance_gate::TranceGate) -> usize {
    trance_gate.step_pos()
}

#[no_mangle]
pub unsafe extern "C" fn process(
    trance_gate: &mut trance_gate::TranceGate,
    inputs: &AudioFrame,
    outputs: &mut AudioFrame,
) {
    trance_gate.process(inputs, outputs);
}

#[no_mangle]
pub unsafe extern "C" fn set_sample_rate(trance_gate: &mut trance_gate::TranceGate, value: f32) {
    trance_gate.set_sample_rate(value);
}

#[no_mangle]
pub unsafe extern "C" fn set_step(
    trance_gate: &mut trance_gate::TranceGate,
    channel: usize,
    step: usize,
    value_normalized: f32,
) {
    trance_gate.set_step(channel, step, value_normalized);
}

#[no_mangle]
pub unsafe extern "C" fn set_width(trance_gate: &mut trance_gate::TranceGate, value: f32) {
    trance_gate.set_width(value);
}

#[no_mangle]
pub unsafe extern "C" fn set_shuffle_amount(trance_gate: &mut trance_gate::TranceGate, value: f32) {
    trance_gate.set_shuffle_amount(value);
}

#[no_mangle]
pub unsafe extern "C" fn set_stereo_mode(trance_gate: &mut trance_gate::TranceGate, value: bool) {
    trance_gate.set_stereo_mode(value);
}

#[no_mangle]
pub unsafe extern "C" fn set_step_len(trance_gate: &mut trance_gate::TranceGate, value: f32) {
    trance_gate.set_step_len(value);
}

#[no_mangle]
pub unsafe extern "C" fn update_project_time_music(
    trance_gate: &mut trance_gate::TranceGate,
    value: f64,
) {
    trance_gate.update_project_time_music(value);
}

#[no_mangle]
pub unsafe extern "C" fn set_step_count(trance_gate: &mut trance_gate::TranceGate, value: usize) {
    trance_gate.set_step_count(value);
}

#[no_mangle]
pub unsafe extern "C" fn set_contour(trance_gate: &mut trance_gate::TranceGate, value_secs: f32) {
    trance_gate.set_contour(value_secs);
}

#[no_mangle]
pub unsafe extern "C" fn set_fade_in(trance_gate: &mut trance_gate::TranceGate, value: f32) {
    trance_gate.set_fade_in(value);
}

#[no_mangle]
pub unsafe extern "C" fn set_delay(trance_gate: &mut trance_gate::TranceGate, value: f32) {
    trance_gate.set_delay(value);
}

#[no_mangle]
pub unsafe extern "C" fn set_mix(trance_gate: &mut trance_gate::TranceGate, value: f32) {
    trance_gate.set_mix(value);
}
