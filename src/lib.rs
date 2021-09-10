// Copyright(c) 2021 Hansen Audio.

pub type RealType = f32;
pub type AudioFrame = [RealType; 4];

pub mod cbindings;
mod detail;
mod trance_gate;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        dsp_tool_box_rs::modulation::phase::Context::new();
        assert_eq!(2 + 2, 4);
    }
}
