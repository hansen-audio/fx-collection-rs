// Copyright(c) 2021 Hansen Audio.

use crate::AudioFrame;

mod delay_line;
mod delay_line_heads;
extern crate dsp_tool_box_rs;
use dsp_tool_box_rs::filtering;

pub struct StereoDelay {
    delay_lines: Vec<delay_line::DelayLine>,
    hi_pass: filtering::one_pole_filter::OnePoleMulti,
    lo_pass: filtering::one_pole_filter::OnePoleMulti,
}

impl StereoDelay {
    const NUM_STEREO_DELAY_CHANNELS: usize = 2;
    const L: usize = 0;
    const R: usize = 1;

    pub fn new() -> Self {
        Self {
            delay_lines: vec![delay_line::DelayLine::new(); Self::NUM_STEREO_DELAY_CHANNELS],
            hi_pass: filtering::one_pole_filter::OnePoleMulti::new(0.),
            lo_pass: filtering::one_pole_filter::OnePoleMulti::new(0.),
        }
    }

    pub fn process(&mut self, inputs: &AudioFrame, outputs: &mut AudioFrame) {
        for (pos, delay_line) in self.delay_lines.iter_mut().enumerate() {
            outputs[pos] = delay_line.process(inputs[pos]);
        }

        *outputs = self.hi_pass.process(outputs);
        *outputs = self.lo_pass.process(outputs);
    }

    pub fn set_normalized_delay_left(&mut self, speed: f32) {
        self.delay_lines[Self::L].set_normalized_delay(speed);
    }

    pub fn set_normalized_delay_right(&mut self, speed: f32) {
        self.delay_lines[Self::R].set_normalized_delay(speed);
    }

    pub fn set_feedback(&mut self, feedback: f32) {
        for el in self.delay_lines.iter_mut() {
            el.set_feedback(feedback);
        }
    }

    pub fn clear_buffer(&mut self) {
        for el in self.delay_lines.iter_mut() {
            el.clear_buffer();
        }
    }

    pub fn set_buffer_size(&mut self, size: usize) {
        for el in self.delay_lines.iter_mut() {
            el.set_buffer_size(size);
        }
    }

    pub fn reset_heads(&mut self) {
        for el in self.delay_lines.iter_mut() {
            el.reset_heads();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delay_line_multi() {
        let inputs = [0. as f32; 4];
        let mut outputs = [0. as f32; 4];
        let mut delay_line = StereoDelay::new();
        delay_line.set_buffer_size(44100);
        delay_line.set_normalized_delay_left(0.5);
        delay_line.set_normalized_delay_right(0.5);
        delay_line.set_feedback(0.5);
        delay_line.clear_buffer();
        delay_line.reset_heads();

        delay_line.process(&inputs, &mut outputs);
    }
}
