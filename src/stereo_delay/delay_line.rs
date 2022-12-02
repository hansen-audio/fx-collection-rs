// Copyright(c) 2022 Hansen Audio.

use dsp_tool_box_rs::filtering;

use super::delay_line_heads;

#[derive(Clone)]
pub(super) struct DelayLine {
    original_buffer: Vec<f32>,
    feedback: f32,
    last_out: f32,
    delay_line_heads: delay_line_heads::DelayLineHeads,
    hp: filtering::one_pole::OnePole,
    lp: filtering::one_pole::OnePole,
}

impl DelayLine {
    pub fn new() -> Self {
        let mut delay_line = Self {
            original_buffer: Vec::new(),
            feedback: 0.75,
            last_out: 0.,
            delay_line_heads: delay_line_heads::DelayLineHeads::new(),
            hp: filtering::one_pole::OnePole::new(),
            lp: filtering::one_pole::OnePole::new(),
        };

        delay_line.original_buffer.resize(8000, 0.);
        delay_line.delay_line_heads.set_buffer_size(8000);
        delay_line
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let read_pos = self.delay_line_heads.read_head();
        self.last_out = self.read_out(read_pos);
        self.last_out = self.filter(self.last_out);

        let current_out = self.last_out;

        self.last_out *= self.feedback;
        self.original_buffer[self.delay_line_heads.write_head()] = input + self.last_out;

        self.delay_line_heads.advance();

        current_out
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

    fn read_out(&mut self, play_back_pos: f32) -> f32 {
        let buffer_size = self.original_buffer.len();
        let mut pos = play_back_pos as usize;
        let fraction = play_back_pos - pos as f32;

        let a = self.original_buffer[pos];
        pos += 1;
        pos = delay_line_heads::DelayLineHeads::bind_to_buffer_usize(pos, buffer_size);
        let b = self.original_buffer[pos];

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
}
