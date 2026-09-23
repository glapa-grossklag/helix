use crate::fold::Folds;
use crate::movement::Direction;
use crate::RopeSlice;

// TODO: switch to std::str::Pattern when it is stable.
pub trait CharMatcher {
    fn char_match(&self, ch: char) -> bool;
}

impl CharMatcher for char {
    fn char_match(&self, ch: char) -> bool {
        *self == ch
    }
}

impl<F: Fn(&char) -> bool> CharMatcher for F {
    fn char_match(&self, ch: char) -> bool {
        (*self)(&ch)
    }
}

// Finds the positions of the nth matching character in given direction
// starting from the pos gap-index (see Range struct for explanation)
//
// The hidden text of a closed fold does not participate in the search (matching vim's
// `f`/`t`, which cannot land inside a closed fold): it is skipped over as if it were not
// there, rather than counting any matches it may contain, and `n` is unaffected by it.
pub fn find_nth_char<M: CharMatcher>(
    mut n: usize,
    text: RopeSlice,
    char_matcher: M,
    mut pos: usize,
    direction: Direction,
    folds: &Folds,
) -> Option<usize> {
    if n == 0 {
        return None;
    }

    // Jumps `pos`/`chars` to the far side of the fold hiding `pos`, in the direction of
    // travel, if any. A no-op when `folds` is empty (the common case), so this changes
    // nothing when there are no folds.
    let skip_fold = |pos: &mut usize, chars: &mut _| -> Option<()> {
        if let Some(fold) = folds.hiding(*pos) {
            *pos = match direction {
                Direction::Forward => fold.end,
                Direction::Backward => fold.start,
            };
            *chars = text.get_chars_at(*pos)?;
        }
        Some(())
    };

    let mut chars = text.get_chars_at(pos)?;
    skip_fold(&mut pos, &mut chars)?;

    match direction {
        Direction::Forward => loop {
            let char_pos = pos;
            let c = chars.next()?;
            pos += 1;
            skip_fold(&mut pos, &mut chars)?;
            if char_matcher.char_match(c) {
                n -= 1;
                if n == 0 {
                    return Some(char_pos);
                }
            }
        },
        Direction::Backward => loop {
            let c = chars.prev()?;
            pos -= 1;
            let char_pos = pos;
            skip_fold(&mut pos, &mut chars)?;
            if char_matcher.char_match(c) {
                n -= 1;
                if n == 0 {
                    return Some(char_pos);
                }
            }
        },
    };
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::movement::Direction;

    #[test]
    fn test_find_nth_char() {
        let text = RopeSlice::from("aa ⌚aa \r\n aa");

        // Forward direction
        assert_eq!(
            find_nth_char(1, text, 'a', 5, Direction::Forward, &Folds::default()),
            Some(5)
        );
        assert_eq!(
            find_nth_char(2, text, 'a', 5, Direction::Forward, &Folds::default()),
            Some(10)
        );
        assert_eq!(
            find_nth_char(3, text, 'a', 5, Direction::Forward, &Folds::default()),
            Some(11)
        );
        assert_eq!(
            find_nth_char(4, text, 'a', 5, Direction::Forward, &Folds::default()),
            None
        );

        // Backward direction
        assert_eq!(
            find_nth_char(1, text, 'a', 5, Direction::Backward, &Folds::default()),
            Some(4)
        );
        assert_eq!(
            find_nth_char(2, text, 'a', 5, Direction::Backward, &Folds::default()),
            Some(1)
        );
        assert_eq!(
            find_nth_char(3, text, 'a', 5, Direction::Backward, &Folds::default()),
            Some(0)
        );
        assert_eq!(
            find_nth_char(4, text, 'a', 5, Direction::Backward, &Folds::default()),
            None
        );

        // Edge cases
        assert_eq!(
            find_nth_char(0, text, 'a', 5, Direction::Forward, &Folds::default()),
            None
        ); // n = 0
        assert_eq!(
            find_nth_char(1, text, 'x', 5, Direction::Forward, &Folds::default()),
            None
        ); // Not found
        assert_eq!(
            find_nth_char(1, text, 'a', 20, Direction::Forward, &Folds::default()),
            None
        ); // Beyond text
        assert_eq!(
            find_nth_char(1, text, 'a', 0, Direction::Backward, &Folds::default()),
            None
        ); // At start going backward
    }
}
