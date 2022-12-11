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
    const TEST_BUF_SIZE: usize = 128;

    const RESULT_LEFT: [f32; TEST_BUF_SIZE] = [
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

    #[test]
    fn test_multi_delay_line_stereo_real_delay() {
        const EXPECTED_RESULT: [f32; TEST_BUF_SIZE] = [
            0.0,
            0.0025263566,
            -0.0025000102,
            -3.2454103e-5,
            5.9776103e-6,
            1.4433031e-7,
            -1.3596773e-8,
            -5.06433e-10,
            2.9069731e-11,
            1.5826152e-12,
            -5.693774e-14,
            -4.592115e-15,
            9.595908e-17,
            1.2602286e-17,
            -1.1100861e-19,
            -3.2996228e-20,
            -6.3649286e-23,
            8.269829e-23,
            1.0232274e-24,
            -1.9825912e-25,
            -4.6526485e-27,
            4.5236377e-28,
            1.6472002e-29,
            -9.7107795e-31,
            -5.174199e-32,
            1.9137489e-33,
            1.5067918e-34,
            -3.2635526e-36,
            -4.147118e-37,
            3.920236e-39,
            1.088614e-39,
            1.449e-42,
            0.0053685103,
            -0.005298962,
            -9.5807656e-5,
            2.563533e-5,
            7.1571e-7,
            -8.861764e-8,
            -3.580167e-9,
            2.5682462e-10,
            1.4598149e-11,
            -6.44173e-13,
            -5.2400684e-14,
            1.362001e-15,
            1.7175736e-16,
            -2.0972974e-18,
            -5.2405304e-19,
            2.52335e-22,
            1.5034041e-21,
            1.5826487e-23,
            -4.0716436e-24,
            -8.900311e-26,
            1.0397854e-26,
            3.606993e-28,
            -2.4847781e-29,
            -1.2640444e-30,
            5.452904e-32,
            4.050207e-33,
            -1.0499005e-34,
            -1.2153827e-35,
            1.5379538e-37,
            3.455686e-38,
            -4.3374e-41,
            -9.3594e-41,
            2.8820903e-5,
            -5.696786e-5,
            2.726627e-5,
            1.0802397e-6,
            -1.9140172e-7,
            -9.043214e-9,
            8.4577684e-10,
            5.226606e-11,
            -2.8921186e-12,
            -2.440349e-13,
            7.9386875e-15,
            9.87947e-16,
            -1.6143374e-17,
            -3.5976537e-18,
            1.1712486e-20,
            1.2026037e-20,
            1.0253992e-22,
            -3.7299458e-23,
            -7.548605e-25,
            1.0774252e-25,
            3.5643216e-27,
            -2.889158e-28,
            -1.4087729e-29,
            7.096152e-31,
            5.0070435e-32,
            -1.5416142e-33,
            -1.6488255e-34,
            2.6736586e-36,
            5.1051716e-37,
            -2.070943e-39,
            -1.497125e-39,
            -1.0651e-41,
            1.5472531e-7,
            -4.5894416e-7,
            4.4703458e-7,
            -1.3475851e-7,
            -9.361547e-9,
            1.2218216e-9,
            8.94821e-11,
            -6.4136274e-12,
            -5.8907084e-13,
            2.4276588e-14,
            3.094128e-15,
            -6.638051e-17,
            -1.3899763e-17,
            9.0102e-20,
            5.5433076e-20,
            3.5213511e-22,
            -2.0038593e-22,
            -3.730101e-24,
            6.635478e-25,
            2.0974505e-26,
            -2.016943e-27,
            -9.471017e-29,
            5.583357e-30,
            3.769429e-31,
            -1.3715665e-32,
            -1.3724272e-33,
            2.7729986e-35,
            4.656027e-36,
            -3.311461e-38,
            -1.4860214e-38,
            -6.8229e-41,
            4.4812e-41,
        ];
        let mut delay_line = StereoDelay::new();
        delay_line.set_buffer_size(32);
        delay_line.set_normalized_delay_left(0.01);
        delay_line.set_normalized_delay_right(0.02);
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
            test_output.push(outputs[StereoDelay::L_CH]);
        }

        //println!("{:#?}", test_output);
        assert_eq!(EXPECTED_RESULT.to_vec(), test_output);
    }
}
