// Copyright(c) 2021 Hansen Audio.

use crate::AudioFrame;

use dsp_tool_box_rs::filtering;

#[derive(Clone, Copy)]
struct DelayLineHeads {
    read_head: f32,
    write_head: usize,
    read_head_increment: f32,
    heads_diff_dst: f32,
    buffer_size: usize,
    buffer_size_f: f32,
}

impl DelayLineHeads {
    pub fn new() -> Self {
        Self {
            read_head: 0.,
            write_head: 0,
            read_head_increment: 0.,
            heads_diff_dst: 0.,
            buffer_size: 8000,
            buffer_size_f: 8000.,
        }
    }

    pub fn advance(&mut self) {
        let tmp_diff = (self.current_diff() - self.heads_diff_dst).abs();
        self.write_head += 1;
        self.read_head = if tmp_diff > 1. {
            self.read_head + self.read_head_increment
        } else {
            self.write_head as f32 - self.heads_diff_dst
        };

        self.read_head = Self::bind_to_buffer(self.read_head, self.buffer_size_f);
        self.write_head = Self::bind_to_buffer(self.write_head, self.buffer_size);
    }

    pub fn reset(&mut self) {
        self.read_head = self.write_head as f32 - self.heads_diff_dst;
        self.read_head_increment = 1.;
        self.read_head = Self::bind_to_buffer(self.read_head, self.buffer_size_f);
    }

    pub fn set_buffer_size(&mut self, buffer_size: usize) {
        self.buffer_size = buffer_size;
        self.buffer_size_f = buffer_size as f32;
    }

    fn calc_read_head_offset(&self, diff: f32) -> f32 {
        diff * self.buffer_size_f
    }

    fn calc_read_head_increment(&mut self, diff: f32) {
        self.heads_diff_dst = self.calc_read_head_offset(diff);
        self.read_head_increment = if self.current_diff() < self.heads_diff_dst {
            0.7
        } else {
            1.3
        };
    }

    pub fn set_heads_diff(&mut self, diff: f32) {
        let _diff = diff.clamp(0., 1.);
        self.calc_read_head_increment(diff);
    }

    fn current_diff(&self) -> f32 {
        let diff = (self.write_head as f32 + self.buffer_size_f) - self.read_head;
        Self::bind_to_buffer(diff, self.buffer_size_f)
    }

    pub fn read_head(&self) -> f32 {
        self.read_head
    }

    pub fn write_head(&self) -> usize {
        self.write_head
    }

    pub fn bind_to_buffer<T>(index: T, buffer_size: T) -> T
    where
        T: Default
            + std::cmp::PartialOrd
            + std::ops::Sub<Output = T>
            + std::ops::Add<Output = T>
            + std::ops::Rem<Output = T>,
    {
        if index < buffer_size {
            index
        } else {
            index % buffer_size
        }
    }
}

#[derive(Clone)]
struct DelayLine {
    original_buffer: Vec<f32>,
    feedback: f32,
    last_out: f32,
    delay_line_heads: DelayLineHeads,
}

impl DelayLine {
    pub fn new() -> Self {
        let mut delay_line = Self {
            original_buffer: Vec::new(),
            feedback: 0.75,
            last_out: 0.,
            delay_line_heads: DelayLineHeads::new(),
        };

        delay_line.original_buffer.resize(8000, 0.);
        delay_line.delay_line_heads.set_buffer_size(8000);
        delay_line
    }

    pub fn process(&mut self, input: f32) -> f32 {
        self.last_out = self.read_out(self.delay_line_heads.read_head()) * self.feedback;
        self.original_buffer[self.delay_line_heads.write_head()] = input + self.last_out;

        self.delay_line_heads.advance();

        self.last_out
    }

    pub fn set_normalized_delay(&mut self, speed: f32) {
        self.delay_line_heads.set_heads_diff(speed)
    }

    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback
    }

    pub fn clear_buffer(&mut self) {
        for item in &mut self.original_buffer {
            *item = 0.;
        }
    }

    pub fn set_buffer_size(&mut self, size: usize) {
        self.original_buffer.resize(size, 0.);
        self.delay_line_heads.set_buffer_size(size);
    }

    pub fn reset_heads(&mut self) {
        self.delay_line_heads.reset()
    }

    fn read_out(&mut self, play_back_pos: f32) -> f32 {
        let buffer_size = self.original_buffer.len();
        let mut pos = play_back_pos as usize;
        let fraction = play_back_pos - pos as f32;

        pos += 1;
        let a = self.original_buffer[pos];
        pos = DelayLineHeads::bind_to_buffer(pos, buffer_size);
        let b = self.original_buffer[pos];

        a + (b - a) * fraction
    }
}

const NUM_STEREO_DELAY_CHANNELS: usize = 2;
const L: usize = 0;
const R: usize = 1;

struct DelayLineStereo {
    delay_lines: Vec<DelayLine>,
    hi_pass: filtering::one_pole_filter::OnePoleMulti,
    lo_pass: filtering::one_pole_filter::OnePoleMulti,
}

impl DelayLineStereo {
    fn new() -> Self {
        Self {
            delay_lines: vec![DelayLine::new(); NUM_STEREO_DELAY_CHANNELS],
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
        self.delay_lines[L].set_normalized_delay(speed);
    }

    pub fn set_normalized_delay_right(&mut self, speed: f32) {
        self.delay_lines[R].set_normalized_delay(speed);
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
    fn name() {
        let inputs = 0. as f32;
        let mut delay_line = DelayLine::new();
        delay_line.set_buffer_size(44100);
        delay_line.set_normalized_delay(0.5);
        delay_line.set_feedback(0.5);
        delay_line.set_normalized_delay(0.5);

        let outputs = delay_line.process(inputs);
        println!("{:?}", outputs);

        delay_line.clear_buffer();
        delay_line.reset_heads();
    }

    #[test]
    fn test_delay_line_multi() {
        let inputs = [0. as f32; 4];
        let mut outputs = [0. as f32; 4];
        let mut delay_line = DelayLineStereo::new();
        delay_line.set_buffer_size(44100);
        delay_line.set_normalized_delay_left(0.5);
        delay_line.set_normalized_delay_right(0.5);
        delay_line.set_feedback(0.5);
        delay_line.clear_buffer();
        delay_line.reset_heads();

        delay_line.process(&inputs, &mut outputs);
    }
}
