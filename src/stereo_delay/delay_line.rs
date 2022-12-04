// Copyright(c) 2022 Hansen Audio.

use super::delay_line_heads::DelayLineHeads;
use dsp_tool_box_rs::filtering::one_pole::OnePole;
use dsp_tool_box_rs::filtering::one_pole::OnePoleType;

#[derive(Clone)]
pub(super) struct DelayLine {
    buffer: Vec<f32>,
    feedback: f32,
    heads: DelayLineHeads,
    hp: OnePole,
    lp: OnePole,
}

impl DelayLine {
    pub fn new() -> Self {
        let mut delay_line = Self {
            buffer: Vec::new(),
            feedback: 0.75,
            heads: DelayLineHeads::new(),
            hp: OnePole::new(),
            lp: OnePole::new(),
        };

        delay_line.hp.set_filter_type(OnePoleType::HP);
        delay_line.lp.set_filter_type(OnePoleType::LP);
        delay_line.buffer.resize(8000, 0.);
        delay_line.heads.set_buffer_size(8000);
        delay_line
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let mut output = self.read(self.heads.read_pos());
        output = self.filter(output);

        let value = input + output * self.feedback;
        self.write(self.heads.write_pos(), value);

        self.heads.advance();

        output
    }

    pub fn set_normalized_delay(&mut self, speed: f32) {
        self.heads.set_heads_diff(speed)
    }

    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback
    }

    pub fn clear_buffer(&mut self) {
        for item in &mut self.buffer {
            *item = 0.;
        }
    }

    pub fn set_buffer_size(&mut self, size: usize) {
        self.buffer.resize(size, 0.);
        self.heads.set_buffer_size(size);
    }

    pub fn reset_heads(&mut self) {
        self.heads.reset()
    }

    pub fn set_lp_freq(&mut self, freq: f32) {
        self.lp.set_frequency(freq);
    }

    pub fn set_hp_freq(&mut self, freq: f32) {
        self.hp.set_frequency(freq);
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.hp.set_sample_rate(sample_rate);
        self.lp.set_sample_rate(sample_rate);
    }

    fn read(&mut self, read_pos: f32) -> f32 {
        let buffer_size = self.buffer.len();
        let mut read_pos_usize = read_pos.floor() as usize;
        let fraction = read_pos.fract();

        let a = self.buffer[read_pos_usize];
        read_pos_usize += 1;
        read_pos_usize = DelayLineHeads::bind_to_buffer_usize(read_pos_usize, buffer_size);
        let b = self.buffer[read_pos_usize];

        a + (b - a) * fraction
    }

    fn write(&mut self, pos: usize, value: f32) {
        self.buffer[pos] = value;
    }

    fn filter(&mut self, input: f32) -> f32 {
        let mut val = self.hp.process_mono(input);
        val = self.lp.process_mono(val);
        val
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup() {
        let inputs = 1. as f32;
        let mut delay_line = DelayLine::new();
        delay_line.set_buffer_size(80);
        delay_line.set_normalized_delay(1.);
        delay_line.set_feedback(0.5);
        delay_line.set_normalized_delay(1.);
        // delay_line.set_hp_freq(freq)

        for _ in 0..800 {
            let outputs = delay_line.process(inputs);
            println!("{:?}", outputs);
        }

        delay_line.clear_buffer();
        delay_line.reset_heads();
    }
}
