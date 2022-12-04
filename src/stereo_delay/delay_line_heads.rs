// Copyright(c) 2022 Hansen Audio.

#[derive(Clone, Copy)]
pub(super) struct DelayLineHeads {
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

        self.read_head = Self::bind_to_buffer_f32(self.read_head, self.buffer_size_f);
        self.write_head = Self::bind_to_buffer_usize(self.write_head, self.buffer_size);
    }

    pub fn reset(&mut self) {
        self.read_head = self.write_head as f32 - self.heads_diff_dst;
        self.read_head_increment = 1.;
        self.read_head = Self::bind_to_buffer_f32(self.read_head, self.buffer_size_f);
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
        Self::bind_to_buffer_f32(diff, self.buffer_size_f)
    }

    pub fn read_pos(&self) -> f32 {
        self.read_head
    }

    pub fn write_pos(&self) -> usize {
        self.write_head
    }

    fn bind_to_buffer_f32(index: f32, buffer_size: f32) -> f32 {
        if index >= buffer_size {
            index - buffer_size
        } else if index < 0 as f32 {
            index + (buffer_size - 1 as f32)
        } else {
            index
        }
    }

    pub fn bind_to_buffer_usize(index: usize, buffer_size: usize) -> usize {
        if index >= buffer_size {
            index - buffer_size
        } else if index < 0 as usize {
            index + (buffer_size - 1 as usize)
        } else {
            index
        }
    }

    /*
    pub fn bind_to_buffer<T>(index: T, buffer_size: T) -> T
    where
        T: Default
            + std::cmp::PartialOrd
            + std::ops::Sub<Output = T>
            + std::ops::Add<Output = T>
            + std::ops::Rem<Output = T>
            + std::ops::Sub<Output = T>,
    {
        if index >= buffer_size {
            index - buffer_size
        } else if index < 0 as T {
            index + buffer_size - (1 as T)
        } else {
            index
        }
    }*/
}
