use std::any::Any;

const NO_MESSAGE: &str = "a panic with no message";

pub fn message(payload: &(dyn Any + Send)) -> String {
    if let Some(said) = payload.downcast_ref::<&str>() {
        return (*said).to_string();
    }
    if let Some(said) = payload.downcast_ref::<String>() {
        return said.clone();
    }
    NO_MESSAGE.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn caught(panicking: impl FnOnce() + std::panic::UnwindSafe) -> String {
        let payload = std::panic::catch_unwind(panicking).expect_err("it panicked");
        message(payload.as_ref())
    }

    #[test]
    fn a_panic_with_a_literal_says_the_literal() {
        assert_eq!(
            caught(|| panic!("index out of range")),
            "index out of range"
        );
    }

    #[test]
    fn a_panic_with_a_formatted_message_says_the_message() {
        let line = 7;
        assert_eq!(
            caught(move || panic!("line {line} is short")),
            "line 7 is short"
        );
    }

    #[test]
    fn a_panic_that_carries_no_text_still_says_something_rather_than_nothing() {
        assert_eq!(
            caught(|| std::panic::panic_any(42_u8)),
            "a panic with no message"
        );
    }
}
