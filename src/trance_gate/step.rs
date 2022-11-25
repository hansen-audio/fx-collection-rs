// Copyright(c) 2022 Hansen Audio.

use super::shuffle_note::is_shuffle_note;

#[derive(Debug, Clone)]
pub struct Step {
    pos: usize,
    count: usize,
    is_shuffle: bool,
}

impl Step {
    pub fn new(pos: usize, count: usize, is_shuffle: bool) -> Self {
        Self {
            pos,
            count,
            is_shuffle,
        }
    }

    pub fn advance(&mut self) {
        self.pos += 1;
        if self.pos >= self.count {
            self.pos = 0;
        }
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn is_shuffle(&self) -> bool {
        self.is_shuffle
    }

    pub fn set_pos(&mut self, pos: usize) {
        self.pos = pos;
    }

    pub fn set_count(&mut self, count: usize) {
        self.count = count;
    }

    pub fn set_note_len(&mut self, note_len: f32) {
        self.is_shuffle = is_shuffle_note(self.pos(), note_len);
    }
}
