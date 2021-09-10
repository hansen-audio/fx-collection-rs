// Copyright(c) 2021 Hansen Audio.

use crate::{trance_gate, AudioFrame, RealType};

//-----------------------------------------------------------------------------
// https://firefox-source-docs.mozilla.org/writing-rust-code/ffi.html
#[no_mangle]
pub unsafe extern "C" fn tg_create() -> *mut trance_gate::Context {
    let tg_context = trance_gate::Context::new();
    Box::into_raw(Box::new(tg_context))
}

#[no_mangle]
pub unsafe extern "C" fn tg_destroy(tg_context: *mut trance_gate::Context) {
    drop(Box::from_raw(tg_context));
}

//-----------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn set_tempo(context: &mut trance_gate::Context, tempo_bpm: RealType) {
    context.set_tempo(tempo_bpm);
}

#[no_mangle]
pub unsafe extern "C" fn trigger(
    context: &mut trance_gate::Context,
    delay_len: RealType,
    fade_in_len: RealType,
) {
    context.trigger(delay_len, fade_in_len);
}

#[no_mangle]
pub unsafe extern "C" fn reset(context: &mut trance_gate::Context) {
    context.reset();
}

#[no_mangle]
pub unsafe extern "C" fn reset_step_pos(context: &mut trance_gate::Context, value: usize) {
    context.reset_step_pos(value);
}

#[no_mangle]
pub unsafe extern "C" fn get_step_pos(context: &mut trance_gate::Context) -> usize {
    context.get_step_pos()
}

#[no_mangle]
pub unsafe extern "C" fn process(
    context: &mut trance_gate::Context,
    inputs: &AudioFrame,
    outputs: &mut AudioFrame,
) {
    context.process(inputs, outputs);
}

#[no_mangle]
pub unsafe extern "C" fn set_sample_rate(context: &mut trance_gate::Context, value: RealType) {
    context.set_sample_rate(value);
}

#[no_mangle]
pub unsafe extern "C" fn set_step(
    context: &mut trance_gate::Context,
    channel: usize,
    step: usize,
    value_normalized: RealType,
) {
    context.set_step(channel, step, value_normalized);
}

#[no_mangle]
pub unsafe extern "C" fn set_width(context: &mut trance_gate::Context, value: RealType) {
    context.set_width(value);
}

#[no_mangle]
pub unsafe extern "C" fn set_shuffle_amount(context: &mut trance_gate::Context, value: RealType) {
    context.set_shuffle_amount(value);
}

#[no_mangle]
pub unsafe extern "C" fn set_stereo_mode(context: &mut trance_gate::Context, value: bool) {
    context.set_stereo_mode(value);
}

#[no_mangle]
pub unsafe extern "C" fn set_step_len(context: &mut trance_gate::Context, value: RealType) {
    context.set_step_len(value);
}

#[no_mangle]
pub unsafe extern "C" fn update_project_time_music(
    context: &mut trance_gate::Context,
    value: RealType,
) {
    context.update_project_time_music(value);
}

#[no_mangle]
pub unsafe extern "C" fn set_step_count(context: &mut trance_gate::Context, value: usize) {
    context.set_step_count(value);
}

#[no_mangle]
pub unsafe extern "C" fn set_contour(context: &mut trance_gate::Context, value_secs: RealType) {
    context.set_contour(value_secs);
}

#[no_mangle]
pub unsafe extern "C" fn set_fade_in(context: &mut trance_gate::Context, value: RealType) {
    context.set_fade_in(value);
}

#[no_mangle]
pub unsafe extern "C" fn set_delay(context: &mut trance_gate::Context, value: RealType) {
    context.set_delay(value);
}

#[no_mangle]
pub unsafe extern "C" fn set_mix(context: &mut trance_gate::Context, value: RealType) {
    context.set_mix(value);
}
