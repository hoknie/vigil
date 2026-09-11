pub const WIDTH: usize = 92;

pub fn comment(paragraph: &str) -> String {
    if paragraph.is_empty() {
        return "#\n".to_string();
    }
    wrap(paragraph, WIDTH)
        .into_iter()
        .map(|line| format!("# {line}\n"))
        .collect()
}

pub fn indented_comment(paragraph: &str) -> String {
    wrap(paragraph, WIDTH - 4)
        .into_iter()
        .map(|line| format!("  # {line}\n"))
        .collect()
}

pub fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        if !line.is_empty() && line.len() + 1 + word.len() > width {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        lines.push(line);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}
