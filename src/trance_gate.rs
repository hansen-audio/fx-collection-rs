// Copyright(c) 2021 Hansen Audio.

use dsp_tool_box_rs as dtb;

use super::detail::shuffle_note::is_shuffle_note;
use crate::NUM_CHANNELS;

#[derive(Debug, Clone)]
struct Step {
    pos: usize,
    count: usize,
    is_shuffle: bool,
}

impl Step {
    fn new(pos: usize, count: usize, is_shuffle: bool) -> Self {
        Self {
            pos,
            count,
            is_shuffle,
        }
    }

    fn advance(&mut self) {
        self.pos += 1;
        if self.pos >= self.count {
            self.pos = 0;
        }
    }
}

const MIN_NUM_STEPS: usize = 1;
const MAX_NUM_STEPS: usize = 32;
const ONE_SAMPLE: usize = 1;
const L: usize = 0;
const R: usize = 1;

type StepVals = [f32; MAX_NUM_STEPS];
type ChannelStepsList = [StepVals; NUM_CHANNELS];
type ContourFiltersList = [dtb::filtering::one_pole_filter::OnePole; NUM_CHANNELS];
type AudioFrame = [f32; NUM_CHANNELS];

#[derive(Debug, Clone)]
pub struct TranceGate {
    channel_steps_list: ChannelStepsList,
    contour_filters: ContourFiltersList,
    delay_phase: dtb::modulation::phase::Phase,
    fade_in_phase: dtb::modulation::phase::Phase,
    step_phase: dtb::modulation::phase::Phase,
    delay_phase_val: f32,
    step_phase_val: f32,
    fade_in_phase_val: f32,
    step_val: Step,
    mix: f32,
    width: f32,
    shuffle: f32,
    contour: f32,
    sample_rate: f32,
    ch: usize,
    is_delay_active: bool,
    is_fade_in_active: bool,
}

impl TranceGate {
    pub fn new() -> Self {
        use dtb::filtering::one_pole_filter::OnePole;
        use dtb::modulation::phase::Phase;
        use dtb::modulation::phase::SyncMode;

        let mut new_self = Self {
            channel_steps_list: [[0.; MAX_NUM_STEPS]; NUM_CHANNELS],
            contour_filters: [OnePole::new(0.9), OnePole::new(0.9)],
            delay_phase: dtb::modulation::phase::Phase::new(),
            fade_in_phase: dtb::modulation::phase::Phase::new(),
            step_phase: dtb::modulation::phase::Phase::new(),
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

        const INIT_NOTE_LEN: f32 = 1. / 32.;

        new_self
            .delay_phase
            .set_rate(Phase::note_len_to_rate(INIT_NOTE_LEN));
        new_self.delay_phase.set_sync_mode(SyncMode::ProjectSync);

        new_self
            .fade_in_phase
            .set_rate(Phase::note_len_to_rate(INIT_NOTE_LEN));
        new_self.fade_in_phase.set_sync_mode(SyncMode::ProjectSync);

        new_self
            .step_phase
            .set_rate(Phase::note_len_to_rate(INIT_NOTE_LEN));
        new_self.step_phase.set_sync_mode(SyncMode::ProjectSync);

        const TEMPO_BPM: f32 = 120.;
        new_self.set_tempo(TEMPO_BPM);

        new_self
    }

    pub fn set_tempo(&mut self, tempo_bpm: f32) {
        self.delay_phase.set_tempo(tempo_bpm);
        self.fade_in_phase.set_tempo(tempo_bpm);
        self.step_phase.set_tempo(tempo_bpm);
    }

    pub fn trigger(&mut self, delay_len: f32, fade_in_len: f32) {
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

    pub fn step_pos(&self) -> usize {
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
        self.step_val.advance();
        set_shuffle(&mut self.step_val, self.step_phase.note_len());
    }

    pub fn set_sample_rate(&mut self, value: f32) {
        use dtb::filtering::one_pole_filter::OnePole;

        self.delay_phase.set_sample_rate(value);
        self.fade_in_phase.set_sample_rate(value);
        self.step_phase.set_sample_rate(value);

        self.sample_rate = value;

        let contour = self.contour;
        self.contour_filters.iter_mut().for_each(|item| {
            let pole = OnePole::tau_to_pole(contour, value);
            item.update_pole(pole);
        });
    }

    pub fn set_step(&mut self, channel: usize, step: usize, value_normalized: f32) {
        self.channel_steps_list[channel][step] = value_normalized;
    }

    pub fn set_width(&mut self, value: f32) {
        self.width = 1. - value;
    }

    pub fn set_shuffle_amount(&mut self, value: f32) {
        self.shuffle = value;
    }

    pub fn set_stereo_mode(&mut self, value: bool) {
        self.ch = match value {
            true => R,
            false => L,
        }
    }

    pub fn set_step_len(&mut self, value: f32) {
        self.step_phase.set_note_len(value);
    }

    pub fn update_project_time_music(&mut self, value: f64) {
        self.delay_phase.set_project_time(value);
        self.fade_in_phase.set_project_time(value);
        self.step_phase.set_project_time(value);
    }

    pub fn set_step_count(&mut self, value: usize) {
        self.step_val.count = value.clamp(MIN_NUM_STEPS, MAX_NUM_STEPS);
    }

    pub fn set_contour(&mut self, value_secs: f32) {
        use dtb::filtering::one_pole_filter::OnePole;

        if self.contour == value_secs {
            return;
        }

        self.contour = value_secs;
        let contour = self.contour;
        let sample_rate = self.sample_rate;
        self.contour_filters.iter_mut().for_each(|item| {
            let pole = OnePole::tau_to_pole(contour, sample_rate);
            item.update_pole(pole);
        });
    }

    pub fn set_fade_in(&mut self, value: f32) {
        self.is_fade_in_active = value > 0.;
        if !self.is_fade_in_active {
            return;
        }

        self.fade_in_phase.set_note_len(value);
    }

    pub fn set_delay(&mut self, value: f32) {
        self.is_delay_active = value > 0.;
        if !self.is_delay_active {
            return;
        }

        self.delay_phase.set_note_len(value);
    }

    pub fn set_mix(&mut self, value: f32) {
        self.mix = value;
    }
}

fn apply_width(value_le: &mut f32, value_ri: &mut f32, width: f32) {
    *value_le = value_le.max(*value_le * width);
    *value_ri = value_ri.max(*value_ri * width);
}

fn apply_mix(value: &mut f32, mix: f32) {
    const MIX_MAX: f32 = 1.;
    *value = (MIX_MAX - mix) + *value * mix;
}

fn apply_mix_stereo(value_le: &mut f32, value_ri: &mut f32, mix: f32) {
    apply_mix(value_le, mix);
    apply_mix(value_ri, mix);
}

fn apply_contour(value_le: &mut f32, value_ri: &mut f32, contour_filters: &mut ContourFiltersList) {
    *value_le = contour_filters[L].process(*value_le);
    *value_ri = contour_filters[R].process(*value_ri);
}

fn apply_gate_delay(value_le: &mut f32, value_ri: &mut f32, phase_value: f32, delay: f32) {
    let factor = match phase_value > delay {
        true => 1.,
        false => 0.,
    };

    *value_le = *value_le * factor;
    *value_ri = *value_ri * factor;
}

fn apply_shuffle(value_le: &mut f32, value_ri: &mut f32, phase_value: f32, context: &TranceGate) {
    // TODO: Is this a good value for a MAX_DELAY?
    const MAX_DELAY: f32 = 3. / 4.;
    let delay = context.shuffle * MAX_DELAY;

    if context.step_val.is_shuffle {
        apply_gate_delay(value_le, value_ri, phase_value, delay);
    }
}

fn set_shuffle(step: &mut Step, note_len: f32) {
    step.is_shuffle = is_shuffle_note(step.pos, note_len);
}

#[cfg(test)]
mod tests {
    use crate::trance_gate::TranceGate;

    #[test]
    #[ignore]
    fn test_tg_context_debug_print() {
        let c = TranceGate::new();
        println!("{:#?}", c);
    }
}
