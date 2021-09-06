// Copyright(c) 2021 Hansen Audio.

use dsp_tool_box_rs;
use dsp_tool_box_rs::filtering::one_pole_filter as contour_filter;
use dsp_tool_box_rs::modulation::phase as mod_phase;

use crate::RealType;

/// A step is represnted by a position, a step count and the shuffle option.
pub struct Step {
    pos: i32,
    count: i32,
    is_shuffle: bool,
}

impl Step {
    pub fn new(pos: i32, count: i32, is_shuffle: bool) -> Self {
        Self {
            pos,
            count,
            is_shuffle,
        }
    }
}

const NUM_CHANNELS_SSE: usize = 4;
const NUM_CHANNELS: usize = 2;
const MIN_NUM_STEPS: i32 = 1;
const MAX_NUM_STEPS: usize = 32;
const L: i32 = 0;
const R: i32 = 1;

type StepValues = [RealType; MAX_NUM_STEPS];
// type ChannelStepsList = [StepValues; NUM_CHANNELS];
type ContourFiltersList = [contour_filter::Context; NUM_CHANNELS];

type AudioFrame = [RealType; NUM_CHANNELS_SSE];

pub struct Context {
    contour_filter: ContourFiltersList,
    delay_phase: mod_phase::Context,
    fade_in_phase: mod_phase::Context,
    step_phase: mod_phase::Context,
    delay_phase_val: RealType,
    step_phase_val: RealType,
    fade_in_phase_val: RealType,
    step_val: Step,
    mix: RealType,
    width: RealType,
    shuffle: RealType,
    contour: RealType,
    sample_rate: RealType,
    ch: i32,
    is_delay_active: bool,
    is_fade_in_active: bool,
}

impl Context {
    pub fn new() -> Self {
        let mut new_obj = Self {
            contour_filter: [
                contour_filter::Context::new(0.9),
                contour_filter::Context::new(0.9),
            ],
            delay_phase: mod_phase::Context::new(),
            fade_in_phase: mod_phase::Context::new(),
            step_phase: mod_phase::Context::new(),
            delay_phase_val: 0.,
            fade_in_phase_val: 0.,
            step_phase_val: 0.,
            step_val: Step::new(0, 32, false),
            mix: 0.,
            width: 0.,
            shuffle: 0.,
            contour: 0.01,
            sample_rate: 44100.,
            ch: 0,
            is_delay_active: false,
            is_fade_in_active: false,
        };

        const INIT_NOTE_LEN: RealType = 1. / 32.;

        new_obj
            .delay_phase
            .set_rate(mod_phase::note_length_to_rate(INIT_NOTE_LEN));
        new_obj
            .delay_phase
            .set_sync_mode(mod_phase::SyncMode::ProjectSync);

        new_obj
            .fade_in_phase
            .set_rate(mod_phase::note_length_to_rate(INIT_NOTE_LEN));
        new_obj
            .fade_in_phase
            .set_sync_mode(mod_phase::SyncMode::ProjectSync);

        new_obj
            .step_phase
            .set_rate(mod_phase::note_length_to_rate(INIT_NOTE_LEN));
        new_obj
            .step_phase
            .set_sync_mode(mod_phase::SyncMode::ProjectSync);

        const TEMPO_BPM: RealType = 120.;
        new_obj.set_tempo(TEMPO_BPM);
        new_obj
    }

    pub fn set_tempo(&mut self, tempo_bpm: RealType) {
        self.delay_phase.set_tempo(tempo_bpm);
    }

    pub fn trigger(&mut self, delay_len: RealType, fade_in_len: RealType) {
        //self.set_delay();
        //self.set_fade_in();

        self.delay_phase_val = 0.;
        self.fade_in_phase_val = 0.;
        self.step_phase_val = 0.;
        self.step_val.pos = 0;

        if self.is_delay_active {
            self.reset();
        }
    }

    pub fn reset(&mut self) {
        let reset_val = match self.is_delay_active {
            true => 1.,
            false => 0.,
        };

        self.contour_filter
            .iter_mut()
            .for_each(|item| item.reset(reset_val));
    }

    pub fn process(&mut self, inputs: &AudioFrame, outputs: &mut AudioFrame) {}
}

#[cfg(test)]
mod tests {
    #[test]
    fn name() {
        println!("");
    }
}
