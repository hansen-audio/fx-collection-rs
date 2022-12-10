// Copyright(c) 2021 Hansen Audio.

use crate::AudioFrame;

mod delay_line_heads;
use delay_line_heads::DelayLineHeads;
use dsp_tool_box_rs::filtering::one_pole::OnePole;
use dsp_tool_box_rs::filtering::one_pole::OnePoleType;

const NUM_STEREO_DELAY_CHANNELS: usize = 2;

#[derive(Clone)]
pub struct StereoDelay {
    bufs: Vec<Vec<f32>>,
    feedbacks: [f32; NUM_STEREO_DELAY_CHANNELS],
    heads: [DelayLineHeads; NUM_STEREO_DELAY_CHANNELS],
    hp: OnePole,
    lp: OnePole,
}

impl StereoDelay {
    const L_CH: usize = 0;
    const R_CH: usize = 1;

    pub fn new() -> Self {
        let mut delay_line = Self {
            bufs: vec![vec![0_f32; 8000]; NUM_STEREO_DELAY_CHANNELS],
            feedbacks: [0.75; NUM_STEREO_DELAY_CHANNELS],
            heads: [DelayLineHeads::new(); NUM_STEREO_DELAY_CHANNELS],
            hp: OnePole::new(),
            lp: OnePole::new(),
        };

        delay_line.hp.set_filter_type(OnePoleType::HP);
        delay_line.lp.set_filter_type(OnePoleType::LP);
        for el in delay_line.heads.iter_mut() {
            el.set_buffer_size(8000);
        }

        delay_line
    }

    pub fn process_mono(&mut self, input: f32) -> f32 {
        let mut output = self.read(Self::L_CH, self.heads[Self::L_CH].read_pos());
        output = self.filter(output);

        let value = input + output * self.feedbacks[Self::L_CH];
        self.write(Self::L_CH, self.heads[Self::L_CH].write_pos(), value);

        self.heads[Self::L_CH].advance();

        output
    }

    pub fn process_stereo(&mut self, outputs: &mut AudioFrame) {
        let inputs = outputs.clone();

        outputs[Self::L_CH] = self.read(Self::L_CH, self.heads[Self::L_CH].read_pos());
        outputs[Self::R_CH] = self.read(Self::R_CH, self.heads[Self::R_CH].read_pos());

        self.filter_multi(outputs);

        let mut value_left = inputs[Self::L_CH];
        value_left += outputs[Self::L_CH] * self.feedbacks[Self::L_CH];
        self.write(Self::L_CH, self.heads[Self::L_CH].write_pos(), value_left);

        let mut value_right = inputs[Self::R_CH];
        value_right += outputs[Self::R_CH] * self.feedbacks[Self::R_CH];
        self.write(Self::R_CH, self.heads[Self::R_CH].write_pos(), value_right);

        for el in self.heads.iter_mut() {
            el.advance();
        }
    }

    pub fn set_normalized_delay_left(&mut self, speed: f32) {
        self.heads[Self::L_CH].set_heads_diff(speed);
    }

    pub fn set_normalized_delay_right(&mut self, speed: f32) {
        self.heads[Self::R_CH].set_heads_diff(speed);
    }

    pub fn set_feedback(&mut self, feedback: f32) {
        for el in self.feedbacks.iter_mut() {
            *el = feedback;
        }
    }

    pub fn clear_buffer(&mut self) {
        for item in &mut self.bufs {
            for v in item.iter_mut() {
                *v = 0_f32;
            }
        }
    }

    pub fn set_buffer_size(&mut self, size: usize) {
        for item in &mut self.bufs {
            (*item).resize(size, 0f32);
        }

        for el in self.heads.iter_mut() {
            el.set_buffer_size(size);
        }
    }

    pub fn reset_heads(&mut self) {
        for el in self.heads.iter_mut() {
            el.reset();
        }
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

    fn read(&self, ch: usize, read_pos: f32) -> f32 {
        let mut buf_pos = read_pos.floor() as usize;
        let a = self.bufs[ch][buf_pos];

        buf_pos = self.heads[ch].increment_pos(buf_pos);
        let b = self.bufs[ch][buf_pos];

        a + (b - a) * read_pos.fract()
    }

    fn write(&mut self, ch: usize, pos: usize, value: f32) {
        self.bufs[ch][pos] = value;
    }

    fn filter(&mut self, input: f32) -> f32 {
        let mut val = self.hp.process_mono(input);
        val = self.lp.process_mono(val);
        val
    }

    fn filter_multi(&mut self, outputs: &mut AudioFrame) {
        self.hp.process(outputs);
        self.lp.process(outputs);
    }
}

#[cfg(test)]
mod tests {
    use crate::NUM_CHANNELS;

    use super::*;

    const RESULT_LEFT: [f32; 128] = [
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.007894864,
        -0.007832478,
        -6.1894214e-5,
        -4.8865047e-7,
        -3.857857e-9,
        -3.0457484e-11,
        -2.4045948e-13,
        -1.8984088e-15,
        -1.4987788e-17,
        -1.1832742e-19,
        -9.341858e-22,
        -7.375324e-24,
        -5.822761e-26,
        -4.597025e-28,
        -3.6293153e-30,
        -2.8653163e-32,
        -2.2621445e-34,
        -1.7859457e-36,
        -1.4099902e-38,
        -1.11316e-40,
        -8.8e-43,
        -7e-45,
        -0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        6.232889e-5,
        -0.00012367271,
        6.0370414e-5,
        9.618544e-7,
        1.14246665e-8,
        1.204415e-10,
        1.1896556e-12,
        1.1277381e-14,
        1.0391712e-16,
        9.379181e-19,
        8.332449e-21,
        7.3107823e-23,
        6.350013e-25,
        5.469776e-27,
        4.6787408e-29,
        3.978359e-31,
        3.3655167e-33,
        2.8343958e-35,
        2.3777472e-37,
        1.987753e-39,
        1.6566e-41,
        1.37e-43,
        1e-45,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        4.9207813e-7,
        -1.4645689e-6,
        1.4414222e-6,
        -4.5763207e-7,
        -1.1119894e-8,
        -1.7709062e-10,
        -2.3402435e-12,
        -2.7786025e-14,
        -3.0764917e-16,
        -3.242515e-18,
        -3.294427e-20,
        -3.2535224e-22,
        -3.1412682e-24,
        -2.97743e-26,
        -2.7791574e-28,
        -2.5606741e-30,
        -2.3333281e-32,
        -2.105836e-34,
        -1.8846272e-36,
        -1.6742106e-38,
        -1.47753e-40,
        -1.296e-42,
        -1.1e-44,
        -0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
    ];

    #[test]
    fn test_multi_delay_line_mono() {
        let mut delay_line = StereoDelay::new();
        delay_line.set_buffer_size(32);
        delay_line.set_normalized_delay_left(0.);
        delay_line.set_feedback(1.);
        delay_line.set_hp_freq(20.);
        delay_line.set_lp_freq(22050.);
        delay_line.reset_heads();
        delay_line.clear_buffer();
        delay_line.set_sample_rate(44100_f32);

        const TEST_BUF_SIZE: usize = 128;
        let mut test_output = Vec::new();

        test_output.push(delay_line.process_mono(1.));
        for _ in 0..(TEST_BUF_SIZE - 1) {
            test_output.push(delay_line.process_mono(0.));
        }

        // println!("{:#?}", test_output);
        assert_eq!(RESULT_LEFT.to_vec(), test_output);
    }

    #[test]
    fn test_multi_delay_line_stereo() {
        let mut delay_line = StereoDelay::new();
        delay_line.set_buffer_size(32);
        delay_line.set_normalized_delay_left(0.);
        delay_line.set_normalized_delay_right(0.);
        delay_line.set_feedback(1.);
        delay_line.set_hp_freq(20.);
        delay_line.set_lp_freq(22050.);
        delay_line.reset_heads();
        delay_line.clear_buffer();
        delay_line.set_sample_rate(44100_f32);

        const TEST_BUF_SIZE: usize = 128;
        let mut test_output = Vec::new();

        let mut outputs: AudioFrame = [1.; NUM_CHANNELS];
        delay_line.process_stereo(&mut outputs);
        test_output.push(outputs[StereoDelay::L_CH]);
        for _ in 0..(TEST_BUF_SIZE - 1) {
            outputs.copy_from_slice(&[0., 0., 0., 0.]);
            delay_line.process_stereo(&mut outputs);
            assert_eq!(outputs[StereoDelay::L_CH], outputs[StereoDelay::R_CH]);
            test_output.push(outputs[StereoDelay::L_CH]);
        }

        //println!("{:#?}", test_output);
        assert_eq!(RESULT_LEFT.to_vec(), test_output);
    }
}
