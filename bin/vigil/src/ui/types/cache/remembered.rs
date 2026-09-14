use std::cell::RefCell;

pub struct Remembered<K, V> {
    held: RefCell<Option<(K, V)>>,
}

impl<K, V> Default for Remembered<K, V> {
    fn default() -> Remembered<K, V> {
        Remembered {
            held: RefCell::new(None),
        }
    }
}

impl<K: PartialEq, V: Clone> Remembered<K, V> {
    pub fn get_or(&self, key: K, compute: impl FnOnce() -> V) -> V {
        {
            let held = self.held.borrow();
            if let Some((asked, answer)) = held.as_ref()
                && *asked == key
            {
                return answer.clone();
            }
        }
        let answer = compute();
        *self.held.borrow_mut() = Some((key, answer.clone()));
        answer
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    #[test]
    fn the_same_question_is_answered_once_and_a_different_one_is_worked_out_again() {
        let remembered: Remembered<(u64, String), usize> = Remembered::default();
        let worked = Cell::new(0);
        let work = |answer: usize| {
            worked.set(worked.get() + 1);
            answer
        };

        assert_eq!(remembered.get_or((1, "users".into()), || work(10)), 10);
        assert_eq!(remembered.get_or((1, "users".into()), || work(99)), 10);
        assert_eq!(
            worked.get(),
            1,
            "a keypress asks for the rows of the list many times, and it is the same list"
        );

        assert_eq!(remembered.get_or((2, "users".into()), || work(20)), 20);
        assert_eq!(
            worked.get(),
            2,
            "a new reading is a new question: an answer about the old one is a stale screen"
        );
    }

    #[test]
    fn working_an_answer_out_may_ask_another_remembered_thing_without_tripping_over_itself() {
        let outer: Remembered<u8, usize> = Remembered::default();
        let inner: Remembered<u8, usize> = Remembered::default();

        let answer = outer.get_or(1, || {
            inner.get_or(1, || 3) + outer.held.borrow().iter().count()
        });

        assert_eq!(answer, 3);
    }
}
