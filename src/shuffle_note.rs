// Copyright(c) 2021 Hansen Audio.

use std::usize;

use crate::RealType;

pub fn is_shuffle_note(node_index: usize, note_len: RealType) -> bool {
    if note_len == 1. / 16. {
        return is_even(node_index + 1, 2);
    } else if note_len == 1. / 32. {
        return is_even(node_index + 2, 4);
    } else if note_len == 1. / 64. {
        return is_even(node_index + 4, 8);
    } else if note_len == 1. / 128. {
        return is_even(node_index + 8, 16);
    } else {
        return false;
    }
}

fn is_odd(value: usize, divider: usize) -> bool {
    value % divider != 0
}

fn is_even(value: usize, divider: usize) -> bool {
    !is_odd(value, divider)
}

#[cfg(test)]
mod tests {
    use crate::{shuffle_note::is_shuffle_note, RealType};

    #[test]
    fn tests_is_shuffle_note_16() {
        let mut step_index = 0;
        const NOTE_LEN: RealType = 1. / 16.;

        assert_eq!(is_shuffle_note(step_index, NOTE_LEN), false);
        step_index += 1;
        assert_eq!(is_shuffle_note(step_index, NOTE_LEN), true);
        step_index += 1;
        assert_eq!(is_shuffle_note(step_index, NOTE_LEN), false);
        step_index += 1;
        assert_eq!(is_shuffle_note(step_index, NOTE_LEN), true);
        step_index += 1;
        assert_eq!(is_shuffle_note(step_index, NOTE_LEN), false);
        step_index += 1;
        assert_eq!(is_shuffle_note(step_index, NOTE_LEN), true);
        step_index += 1;
        assert_eq!(is_shuffle_note(step_index, NOTE_LEN), false);
        step_index += 1;
        assert_eq!(is_shuffle_note(step_index, NOTE_LEN), true);
    }

    #[test]
    fn tests_is_shuffle_note_32() {
        let mut step_index = 0;
        const NOTE_LEN: RealType = 1. / 32.;

        assert_eq!(is_shuffle_note(step_index, NOTE_LEN), false);
        step_index += 1;
        assert_eq!(is_shuffle_note(step_index, NOTE_LEN), false);
        step_index += 1;
        assert_eq!(is_shuffle_note(step_index, NOTE_LEN), true);
        step_index += 1;
        assert_eq!(is_shuffle_note(step_index, NOTE_LEN), false);
        step_index += 1;
        assert_eq!(is_shuffle_note(step_index, NOTE_LEN), false);
        step_index += 1;
        assert_eq!(is_shuffle_note(step_index, NOTE_LEN), false);
        step_index += 1;
        assert_eq!(is_shuffle_note(step_index, NOTE_LEN), true);
        step_index += 1;
        assert_eq!(is_shuffle_note(step_index, NOTE_LEN), false);
        step_index += 1;
        assert_eq!(is_shuffle_note(step_index, NOTE_LEN), false);
        step_index += 1;
        assert_eq!(is_shuffle_note(step_index, NOTE_LEN), false);
        step_index += 1;
        assert_eq!(is_shuffle_note(step_index, NOTE_LEN), true);
    }
}
