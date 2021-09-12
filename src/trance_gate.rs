// Copyright(c) 2021 Hansen Audio.

use dsp_tool_box_rs;

use crate::detail::shuffle_note::is_shuffle_note;
use crate::RealType;

/// A step is represented by a position, a step count and the shuffle option.
// #[repr(C)]
pub struct Step {
    pos: usize,
    count: usize,
    is_shuffle: bool,
}

impl Step {
    pub fn new(pos: usize, count: usize, is_shuffle: bool) -> Self {
        Self {
            pos,
            count,
            is_shuffle,
        }
    }

    pub fn inc(&mut self) {
        self.pos = self.pos + 1;
        if self.pos >= self.count {
            self.pos = 0;
        }
    }
}

const NUM_CHANNELS_SSE: usize = 4;
const NUM_CHANNELS: usize = 2;
const MIN_NUM_STEPS: usize = 1;
const MAX_NUM_STEPS: usize = 32;
const L: usize = 0;
const R: usize = 1;
const ONE_SAMPLE: usize = 1;

type StepValues = [RealType; MAX_NUM_STEPS];
type ChannelStepsList = [StepValues; NUM_CHANNELS];
type ContourFiltersList =
    [dsp_tool_box_rs::filtering::one_pole_filter::OnePoleContext; NUM_CHANNELS];
type AudioFrame = [RealType; NUM_CHANNELS_SSE];

// #[repr(C)]
pub struct Context {
    channel_steps_list: ChannelStepsList,
    contour_filters: ContourFiltersList,
    delay_phase: dsp_tool_box_rs::modulation::phase::PhaseContext,
    fade_in_phase: dsp_tool_box_rs::modulation::phase::PhaseContext,
    step_phase: dsp_tool_box_rs::modulation::phase::PhaseContext,
    delay_phase_val: RealType,
    step_phase_val: RealType,
    fade_in_phase_val: RealType,
    step_val: Step,
    mix: RealType,
    width: RealType,
    shuffle: RealType,
    contour: RealType,
    sample_rate: RealType,
    ch: usize,
    is_delay_active: bool,
    is_fade_in_active: bool,
}

impl Context {
    pub fn new() -> Self {
        let mut new_self = Self {
            channel_steps_list: [[0.; MAX_NUM_STEPS]; NUM_CHANNELS],
            contour_filters: [
                dsp_tool_box_rs::filtering::one_pole_filter::OnePoleContext::new(0.9),
                dsp_tool_box_rs::filtering::one_pole_filter::OnePoleContext::new(0.9),
            ],
            delay_phase: dsp_tool_box_rs::modulation::phase::PhaseContext::new(),
            fade_in_phase: dsp_tool_box_rs::modulation::phase::PhaseContext::new(),
            step_phase: dsp_tool_box_rs::modulation::phase::PhaseContext::new(),
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

        new_self
            .delay_phase
            .set_rate(dsp_tool_box_rs::modulation::phase::note_length_to_rate(
                INIT_NOTE_LEN,
            ));
        new_self
            .delay_phase
            .set_sync_mode(dsp_tool_box_rs::modulation::phase::SyncMode::ProjectSync);

        new_self
            .fade_in_phase
            .set_rate(dsp_tool_box_rs::modulation::phase::note_length_to_rate(
                INIT_NOTE_LEN,
            ));
        new_self
            .fade_in_phase
            .set_sync_mode(dsp_tool_box_rs::modulation::phase::SyncMode::ProjectSync);

        new_self
            .step_phase
            .set_rate(dsp_tool_box_rs::modulation::phase::note_length_to_rate(
                INIT_NOTE_LEN,
            ));
        new_self
            .step_phase
            .set_sync_mode(dsp_tool_box_rs::modulation::phase::SyncMode::ProjectSync);

        const TEMPO_BPM: RealType = 120.;
        new_self.set_tempo(TEMPO_BPM);

        new_self
    }

    pub fn set_tempo(&mut self, tempo_bpm: RealType) {
        self.delay_phase.set_tempo(tempo_bpm);
        self.fade_in_phase.set_tempo(tempo_bpm);
        self.step_phase.set_tempo(tempo_bpm);
    }

    pub fn trigger(&mut self, delay_len: RealType, fade_in_len: RealType) {
        self.set_delay(delay_len);
        self.set_fade_in(fade_in_len);

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

        self.contour_filters
            .iter_mut()
            .for_each(|item| item.reset(reset_val));
    }

    pub fn reset_step_pos(&mut self, value: usize) {
        self.step_val.pos = value;
    }

    pub fn get_step_pos(&mut self) -> usize {
        self.step_val.pos
    }

    pub fn process(&mut self, inputs: &AudioFrame, outputs: &mut AudioFrame) {
        let is_overflow = self
            .delay_phase
            .advance_one_shot(&mut self.delay_phase_val, ONE_SAMPLE);

        if !is_overflow {
            outputs.copy_from_slice(inputs);
            return;
        }

        let mut value_le = self.channel_steps_list[L][self.step_val.pos];
        let mut value_ri = self.channel_steps_list[self.ch][self.step_val.pos];

        apply_shuffle(&mut value_le, &mut value_ri, self.step_phase_val, self);
        apply_width(&mut value_le, &mut value_ri, self.width);
        apply_contour(&mut value_le, &mut value_ri, &mut self.contour_filters);
        let tmp_mix = match self.is_fade_in_active {
            true => self.mix * self.fade_in_phase_val,
            false => self.mix,
        };
        apply_mix_stereo(&mut value_le, &mut value_ri, tmp_mix);

        outputs[L] = inputs[L] * value_le;
        outputs[R] = inputs[R] * value_ri;

        self.update_phases()
    }

    fn update_phases(&mut self) {
        self.fade_in_phase
            .advance_one_shot(&mut self.fade_in_phase_val, ONE_SAMPLE);

        let is_overflow = self
            .step_phase
            .advance(&mut self.step_phase_val, ONE_SAMPLE);

        if !is_overflow {
            return;
        }
        self.step_val.inc();
        set_shuffle(&mut self.step_val, self.step_phase.get_note_len());
    }

    pub fn set_sample_rate(&mut self, value: RealType) {
        self.delay_phase.set_sample_rate(value);
        self.fade_in_phase.set_sample_rate(value);
        self.step_phase.set_sample_rate(value);

        self.sample_rate = value;

        let contour = self.contour;
        self.contour_filters.iter_mut().for_each(|item| {
            let pole = dsp_tool_box_rs::filtering::one_pole_filter::tau_to_pole(contour, value);
            item.update_pole(pole);
        });
    }

    pub fn set_step(&mut self, channel: usize, step: usize, value_normalized: RealType) {
        self.channel_steps_list[channel][step] = value_normalized;
    }

    pub fn set_width(&mut self, value: RealType) {
        self.width = 1. - value;
    }

    pub fn set_shuffle_amount(&mut self, value: RealType) {
        self.shuffle = value;
    }

    pub fn set_stereo_mode(&mut self, value: bool) {
        self.ch = match value {
            true => R,
            false => L,
        }
    }

    pub fn set_step_len(&mut self, value: RealType) {
        self.step_phase.set_note_len(value);
    }

    pub fn update_project_time_music(&mut self, value: RealType) {
        self.delay_phase.set_project_time(value);
        self.fade_in_phase.set_project_time(value);
        self.step_phase.set_project_time(value);
    }

    pub fn set_step_count(&mut self, value: usize) {
        self.step_val.count = value.clamp(MIN_NUM_STEPS, MAX_NUM_STEPS);
    }

    pub fn set_contour(&mut self, value_secs: RealType) {
        if self.contour == value_secs {
            return;
        }

        self.contour = value_secs;
        let contour = self.contour;
        let sample_rate = self.sample_rate;
        self.contour_filters.iter_mut().for_each(|item| {
            let pole =
                dsp_tool_box_rs::filtering::one_pole_filter::tau_to_pole(contour, sample_rate);
            item.update_pole(pole);
        });
    }

    pub fn set_fade_in(&mut self, value: RealType) {
        self.is_fade_in_active = value > 0.;
        if !self.is_fade_in_active {
            return;
        }

        self.fade_in_phase.set_note_len(value);
    }

    pub fn set_delay(&mut self, value: RealType) {
        self.is_delay_active = value > 0.;
        if !self.is_delay_active {
            return;
        }

        self.delay_phase.set_note_len(value);
    }

    pub fn set_mix(&mut self, value: RealType) {
        self.mix = value;
    }
}

fn apply_width(value_le: &mut RealType, value_ri: &mut RealType, width: RealType) {
    *value_le = value_le.max(*value_le * width);
    *value_ri = value_ri.max(*value_ri * width);
}

fn apply_mix(value: &mut RealType, mix: RealType) {
    const MIX_MAX: RealType = 1.;
    *value = (MIX_MAX - mix) + *value * mix;
}

fn apply_mix_stereo(value_le: &mut RealType, value_ri: &mut RealType, mix: RealType) {
    apply_mix(value_le, mix);
    apply_mix(value_ri, mix);
}

fn apply_contour(
    value_le: &mut RealType,
    value_ri: &mut RealType,
    contour_filters: &mut ContourFiltersList,
) {
    *value_le = contour_filters[L].process(*value_le);
    *value_ri = contour_filters[R].process(*value_ri);
}

fn apply_gate_delay(
    value_le: &mut RealType,
    value_ri: &mut RealType,
    phase_value: RealType,
    delay: RealType,
) {
    let factor = match phase_value > delay {
        true => 1.,
        false => 0.,
    };

    *value_le = *value_le * factor;
    *value_ri = *value_ri * factor;
}

fn apply_shuffle(
    value_le: &mut RealType,
    value_ri: &mut RealType,
    phase_value: RealType,
    context: &Context,
) {
    // TODO: Is this a good value for a MAX_DELAY?
    const MAX_DELAY: RealType = 3. / 4.;
    let delay = context.shuffle * MAX_DELAY;

    if context.step_val.is_shuffle {
        apply_gate_delay(value_le, value_ri, phase_value, delay);
    }
}

fn set_shuffle(step: &mut Step, note_len: RealType) {
    let pos = step.pos;
    step.is_shuffle = is_shuffle_note(pos, note_len);
}

#[cfg(test)]
mod tests {
    #[test]
    fn name() {
        println!("");
    }
}
