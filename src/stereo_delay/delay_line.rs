// Copyright(c) 2022 Hansen Audio.

use super::delay_line_heads;

#[derive(Clone)]
pub(super) struct DelayLine {
    original_buffer: Vec<f32>,
    feedback: f32,
    last_out: f32,
    delay_line_heads: delay_line_heads::DelayLineHeads,
}

impl DelayLine {
    pub fn new() -> Self {
        let mut delay_line = Self {
            original_buffer: Vec::new(),
            feedback: 0.75,
            last_out: 0.,
            delay_line_heads: delay_line_heads::DelayLineHeads::new(),
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
        pos = delay_line_heads::DelayLineHeads::bind_to_buffer(pos, buffer_size);
        let b = self.original_buffer[pos];

        a + (b - a) * fraction
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
