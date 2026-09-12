pub fn unescaped(field: &str) -> String {
    let mut text = String::with_capacity(field.len());
    let mut characters = field.chars();

    while let Some(character) = characters.next() {
        if character != '\\' {
            text.push(character);
            continue;
        }
        let digits: String = characters.clone().take(3).collect();
        match u8::from_str_radix(&digits, 8) {
            Ok(byte) if digits.len() == 3 => {
                text.push(char::from(byte));
                characters.nth(2);
            }
            _ => text.push(character),
        }
    }

    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_octal_the_kernel_writes_for_a_space_is_read_back_as_the_space() {
        assert_eq!(unescaped("/mnt/my\\040disk"), "/mnt/my disk");
        assert_eq!(unescaped("/mnt/a\\011tab"), "/mnt/a\ttab");
        assert_eq!(unescaped("/mnt/plain"), "/mnt/plain");
    }

    #[test]
    fn a_backslash_that_is_not_an_escape_is_the_backslash_that_was_there() {
        assert_eq!(unescaped("/mnt/back\\slash"), "/mnt/back\\slash");
        assert_eq!(unescaped("/mnt/half\\04"), "/mnt/half\\04");
        assert_eq!(unescaped("\\"), "\\");
    }
}
