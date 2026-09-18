const SEPARATOR: &str = "---";

pub struct Document {
    pub name: Option<String>,
    pub text: String,
}

pub fn documents(text: &str) -> Vec<Document> {
    let mut documents = Vec::new();
    let mut held = String::new();
    for line in text.split_inclusive('\n') {
        if line.trim_end() == SEPARATOR {
            documents.push(document(std::mem::take(&mut held)));
            continue;
        }
        held.push_str(line);
    }
    documents.push(document(held));
    documents
        .into_iter()
        .filter(|document| !document.text.trim().is_empty())
        .collect()
}

pub fn names(text: &str) -> Vec<String> {
    documents(text)
        .into_iter()
        .filter_map(|document| document.name)
        .collect()
}

pub fn kept(text: &str, runs: impl Fn(&str) -> bool) -> Option<String> {
    let kept: Vec<String> = documents(text)
        .into_iter()
        .filter(|document| document.name.as_deref().is_some_and(&runs))
        .map(|document| document.text)
        .collect();
    let mut joined = String::new();
    for (index, document) in kept.iter().enumerate() {
        if index > 0 {
            if !joined.ends_with('\n') {
                joined.push('\n');
            }
            joined.push_str(SEPARATOR);
            joined.push('\n');
        }
        joined.push_str(document);
    }
    match kept.is_empty() {
        true => None,
        false => Some(joined),
    }
}

fn document(text: String) -> Document {
    let name = text
        .lines()
        .find(|line| !line.is_empty() && !line.starts_with([' ', '#']))
        .and_then(|line| line.split_once(':'))
        .map(|(name, _)| name.trim().to_string());
    Document { name, text }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TWO: &str =
        "# the first\nfirst:\n  schedule: 1\n---\n# the second\nsecond:\n  schedule: 2\n";

    #[test]
    fn each_document_of_a_file_is_named_after_the_collector_whose_block_it_holds() {
        assert_eq!(names(TWO), vec!["first".to_string(), "second".to_string()]);
    }

    #[test]
    fn a_file_of_two_collectors_keeps_only_the_block_of_the_one_that_runs_here() {
        let kept = kept(TWO, |name| name == "second").expect("one runs");

        assert_eq!(kept, "# the second\nsecond:\n  schedule: 2\n");
    }

    #[test]
    fn a_file_whose_collectors_all_run_here_is_written_as_it_was_shipped() {
        assert_eq!(kept(TWO, |_| true).as_deref(), Some(TWO));
    }

    #[test]
    fn a_file_is_written_byte_for_byte_even_when_its_last_line_has_no_newline() {
        let text = TWO.trim_end();

        assert_eq!(kept(text, |_| true).as_deref(), Some(text));
    }

    #[test]
    fn a_file_none_of_whose_collectors_runs_here_is_no_file_at_all() {
        assert_eq!(kept(TWO, |_| false), None);
    }
}
