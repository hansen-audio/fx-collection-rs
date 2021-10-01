// Copyright(c) 2021 Hansen Audio.

use crate::Real;

pub fn is_shuffle_note(note_index: usize, note_len: Real) -> bool {
    if note_len == 1. / 16. {
        return is_even(note_index + 1, 2);
    } else if note_len == 1. / 32. {
        return is_even(note_index + 2, 4);
    } else if note_len == 1. / 64. {
        return is_even(note_index + 4, 8);
    } else if note_len == 1. / 128. {
        return is_even(note_index + 8, 16);
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
    use super::*;

    #[test]
    fn tests_is_shuffle_note_16() {
        let mut step_index = 0;
        const NOTE_LEN: Real = 1. / 16.;
        const TEST_RESULTS: [bool; 8] = [false, true, false, true, false, true, false, true];
        for r in TEST_RESULTS {
            assert_eq!(is_shuffle_note(step_index, NOTE_LEN), r);
            step_index += 1;
        }
    }

    #[test]
    fn tests_is_shuffle_note_32() {
        let mut step_index = 0;
        const NOTE_LEN: Real = 1. / 32.;
        const TEST_RESULTS: [bool; 11] = [
            false, false, true, false, false, false, true, false, false, false, true,
        ];

        for r in TEST_RESULTS {
            assert_eq!(is_shuffle_note(step_index, NOTE_LEN), r);
            step_index += 1;
        }
    }
}
