// Copyright(c) 2021 Hansen Audio.

use dsp_tool_box_rs::filtering;
use dsp_tool_box_rs::modulation;

mod shuffle_note;
mod step;

use crate::AudioFrame;
use crate::NUM_CHANNELS;

const MAX_NUM_STEPS: usize = 32;
type StepVals = [f32; MAX_NUM_STEPS];
type ChannelStepsList = [StepVals; NUM_CHANNELS];

#[derive(Debug, Clone)]
pub struct TranceGate {
    channel_steps_list: ChannelStepsList,
    contour_filter: filtering::one_pole_filter::OnePoleMulti,
    delay_phase: modulation::phase::Phase,
    fade_in_phase: modulation::phase::Phase,
    step_phase: modulation::phase::Phase,
    delay_phase_val: f32,
    step_phase_val: f32,
    fade_in_phase_val: f32,
    step_val: step::Step,
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
    const L: usize = 0;
    const R: usize = 1;
    const MIN_NUM_STEPS: usize = 1;
    const ONE_SAMPLE: usize = 1;

    pub fn new() -> Self {
        use filtering::one_pole_filter::OnePoleMulti;
        use modulation::phase::Phase;
        use modulation::phase::SyncMode;

        let mut trance_gate = Self {
            channel_steps_list: [[0.; MAX_NUM_STEPS]; NUM_CHANNELS],
            contour_filter: OnePoleMulti::new(0.),
            delay_phase: modulation::phase::Phase::new(),
            fade_in_phase: modulation::phase::Phase::new(),
            step_phase: modulation::phase::Phase::new(),
            delay_phase_val: 0.,
            fade_in_phase_val: 0.,
            step_phase_val: 0.,
            step_val: step::Step::new(0, 32, false),
            mix: 0.5,
            width: 0.,
            shuffle: 0.,
            contour: 0.01,
            sample_rate: 44100.,
            ch: 0,
            is_delay_active: false,
            is_fade_in_active: false,
        };

        const INIT_NOTE_LEN: f32 = 1. / 32.;

        trance_gate
            .delay_phase
            .set_rate(Phase::note_len_to_rate(INIT_NOTE_LEN));
        trance_gate.delay_phase.set_sync_mode(SyncMode::ProjectSync);

        trance_gate
            .fade_in_phase
            .set_rate(Phase::note_len_to_rate(INIT_NOTE_LEN));
        trance_gate
            .fade_in_phase
            .set_sync_mode(SyncMode::ProjectSync);

        trance_gate
            .step_phase
            .set_rate(Phase::note_len_to_rate(INIT_NOTE_LEN));
        trance_gate.step_phase.set_sync_mode(SyncMode::ProjectSync);

        const TEMPO_BPM: f32 = 120.;
        trance_gate.set_tempo(TEMPO_BPM);

        trance_gate
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
        self.step_val.set_pos(0);

        if self.is_delay_active {
            self.reset();
        }
    }

    pub fn reset(&mut self) {
        let reset_val = match self.is_delay_active {
            true => 1.,
            false => 0.,
        };

        self.contour_filter.reset(reset_val);
    }

    pub fn reset_step_pos(&mut self, step_pos: usize) {
        self.step_val.set_pos(step_pos);
    }

    pub fn step_pos(&self) -> usize {
        self.step_val.pos()
    }

    pub fn process(&mut self, inputs: &AudioFrame, outputs: &mut AudioFrame) {
        if self.is_delay_running() {
            outputs.copy_from_slice(inputs);
            return;
        }

        let pos = self.step_val.pos();
        let mut left = self.channel_steps_list[Self::L][pos];
        let mut right = self.channel_steps_list[self.ch][pos];

        self.apply_effect(&mut left, &mut right);

        outputs[Self::L] = inputs[Self::L] * left;
        outputs[Self::R] = inputs[Self::R] * right;

        self.update_phases()
    }

    fn update_phases(&mut self) {
        self.fade_in_phase
            .advance_one_shot(&mut self.fade_in_phase_val, Self::ONE_SAMPLE);

        let is_overflow = self
            .step_phase
            .advance(&mut self.step_phase_val, Self::ONE_SAMPLE);

        if !is_overflow {
            return;
        }
        self.step_val.advance();
        Self::set_shuffle(&mut self.step_val, self.step_phase.note_len());
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;

        self.delay_phase.set_sample_rate(sample_rate);
        self.fade_in_phase.set_sample_rate(sample_rate);
        self.step_phase.set_sample_rate(sample_rate);

        self.update_filter_poles();
    }

    pub fn set_step(&mut self, channel: usize, step: usize, value_normalized: f32) {
        self.channel_steps_list[channel][step] = value_normalized;
    }

    pub fn set_width(&mut self, width: f32) {
        self.width = 1. - width;
    }

    pub fn set_shuffle_amount(&mut self, shuffle: f32) {
        self.shuffle = shuffle;
    }

    pub fn set_stereo_mode(&mut self, mode: bool) {
        self.ch = match mode {
            true => Self::R,
            false => Self::L,
        }
    }

    pub fn set_step_len(&mut self, step_len: f32) {
        self.step_phase.set_note_len(step_len);
    }

    pub fn update_project_time_music(&mut self, project_time_music: f64) {
        self.delay_phase.set_project_time(project_time_music);
        self.fade_in_phase.set_project_time(project_time_music);
        self.step_phase.set_project_time(project_time_music);
    }

    pub fn set_step_count(&mut self, step_count: usize) {
        self.step_val
            .set_count(step_count.clamp(Self::MIN_NUM_STEPS, MAX_NUM_STEPS));
    }

    pub fn set_contour(&mut self, contour: f32) {
        if self.contour == contour {
            return;
        }

        self.contour = contour;
        self.update_filter_poles();
    }

    pub fn set_fade_in(&mut self, fade_in: f32) {
        self.is_fade_in_active = fade_in > 0.;
        if !self.is_fade_in_active {
            return;
        }

        self.fade_in_phase.set_note_len(fade_in);
    }

    pub fn set_delay(&mut self, delay: f32) {
        self.is_delay_active = delay > 0.;
        if !self.is_delay_active {
            return;
        }

        self.delay_phase.set_note_len(delay);
    }

    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix;
    }

    // private
    fn update_filter_poles(&mut self) {
        self.contour_filter.set_tau(self.contour, self.sample_rate);
    }

    fn is_delay_running(&mut self) -> bool {
        let is_overflow = self
            .delay_phase
            .advance_one_shot(&mut self.delay_phase_val, Self::ONE_SAMPLE);

        !is_overflow && self.is_delay_active
    }

    fn apply_effect(&mut self, left: &mut f32, right: &mut f32) {
        self.apply_shuffle(left, right);
        self.apply_width(left, right);
        self.apply_contour(left, right);
        self.apply_mix_stereo(left, right);
    }

    fn apply_shuffle(&mut self, left: &mut f32, right: &mut f32) {
        // TODO: Is this a good value for a MAX_DELAY?
        const MAX_DELAY: f32 = 3. / 4.;
        let delay = self.shuffle * MAX_DELAY;

        if self.step_val.is_shuffle() {
            Self::apply_gate_delay(left, right, self.step_phase_val, delay);
        }
    }

    fn apply_width(&self, left: &mut f32, right: &mut f32) {
        *left = left.max(*left * self.width);
        *right = right.max(*right * self.width);
    }

    fn apply_contour(&mut self, left: &mut f32, right: &mut f32) {
        let input = [*left, *right, 0., 0.];
        let output = self.contour_filter.process(&input);
        *left = output[Self::L];
        *right = output[Self::R];
    }

    fn compute_mix(&self) -> f32 {
        match self.is_fade_in_active {
            true => self.mix * self.fade_in_phase_val,
            false => self.mix,
        }
    }

    fn apply_mix(value: &mut f32, mix: f32) {
        const MIX_MAX: f32 = 1.;
        *value = (MIX_MAX - mix) + *value * mix;
    }

    fn apply_mix_stereo(&self, left: &mut f32, right: &mut f32) {
        let mix = self.compute_mix();
        Self::apply_mix(left, mix);
        Self::apply_mix(right, mix);
    }

    fn apply_gate_delay(left: &mut f32, right: &mut f32, phase_value: f32, delay: f32) {
        let factor = match phase_value > delay {
            true => 1.,
            false => 0.,
        };

        *left *= factor;
        *right *= factor;
    }

    fn set_shuffle(step: &mut step::Step, note_len: f32) {
        step.set_note_len(note_len);
    }
}
