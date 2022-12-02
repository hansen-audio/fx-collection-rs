// Copyright(c) 2022 Hansen Audio.

use super::delay_line_heads::DelayLineHeads;
use dsp_tool_box_rs::filtering::one_pole::OnePole;
use dsp_tool_box_rs::filtering::one_pole::OnePoleType;

#[derive(Clone)]
pub(super) struct DelayLine {
    original_buffer: Vec<f32>,
    feedback: f32,
    last_out: f32,
    heads: DelayLineHeads,
    hp: OnePole,
    lp: OnePole,
}

impl DelayLine {
    pub fn new() -> Self {
        let mut delay_line = Self {
            original_buffer: Vec::new(),
            feedback: 0.75,
            last_out: 0.,
            heads: DelayLineHeads::new(),
            hp: OnePole::new(),
            lp: OnePole::new(),
        };

        delay_line.hp.set_filter_type(OnePoleType::HP);
        delay_line.lp.set_filter_type(OnePoleType::LP);
        delay_line.original_buffer.resize(8000, 0.);
        delay_line.heads.set_buffer_size(8000);
        delay_line
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let read_pos = self.heads.read_head();
        self.last_out = self.read_out(read_pos);
        self.last_out = self.filter(self.last_out);

        let current_out = self.last_out;

        self.last_out *= self.feedback;

        let write_pos = self.heads.write_head();
        self.original_buffer[write_pos] = input + self.last_out;

        self.heads.advance();

        current_out
    }

    pub fn set_normalized_delay(&mut self, speed: f32) {
        self.heads.set_heads_diff(speed)
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

    fn read_out(&mut self, read_pos: f32) -> f32 {
        let buffer_size = self.original_buffer.len();
        let mut read_pos_usize = read_pos as usize;
        let fraction = read_pos - read_pos_usize as f32;

        let a = self.original_buffer[read_pos_usize];
        read_pos_usize += 1;
        read_pos_usize = DelayLineHeads::bind_to_buffer_usize(read_pos_usize, buffer_size);
        let b = self.original_buffer[read_pos_usize];

        a + (b - a) * fraction
    }

    fn filter(&mut self, input: f32) -> f32 {
        self.lp.process_mono(self.hp.process_mono(input))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup() {
        let inputs = 0. as f32;
        let mut delay_line = DelayLine::new();
        delay_line.set_buffer_size(44100);
        delay_line.set_normalized_delay(0.5);
        delay_line.set_feedback(0.5);
        delay_line.set_normalized_delay(0.5);
        // delay_line.set_hp_freq(freq)

        for _ in 0..1000 {
            let _outputs = delay_line.process(inputs);
            //println!("{:?}", outputs);
        }

        delay_line.clear_buffer();
        delay_line.reset_heads();
    }
}
