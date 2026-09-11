use clap::builder::Styles;
use clap::builder::styling::AnsiColor;

pub const HELP: Styles = Styles::styled()
    .header(AnsiColor::Cyan.on_default().bold())
    .usage(AnsiColor::Cyan.on_default().bold())
    .literal(AnsiColor::White.on_default().bold())
    .placeholder(AnsiColor::Cyan.on_default())
    .valid(AnsiColor::Green.on_default())
    .invalid(AnsiColor::Red.on_default().bold())
    .error(AnsiColor::Red.on_default().bold());

#[cfg(test)]
mod tests {
    use clap::CommandFactory;
    use clap::Parser;

    use crate::cli::Cli;

    fn stripped(text: &str) -> String {
        let mut plain = String::with_capacity(text.len());
        let mut characters = text.chars();
        while let Some(character) = characters.next() {
            match character {
                '\u{1b}' => {
                    for skipped in characters.by_ref() {
                        if skipped.is_ascii_alphabetic() {
                            break;
                        }
                    }
                }
                _ => plain.push(character),
            }
        }
        plain
    }

    #[test]
    fn what_the_colour_adds_to_the_help_is_only_colour() {
        let help = Cli::command().render_long_help();

        let coloured = help.ansi().to_string();
        let plain = help.to_string();

        assert!(
            coloured.contains('\u{1b}'),
            "the styles are not reaching the page at all"
        );
        assert!(
            !plain.contains('\u{1b}'),
            "what is printed to a pipe or under NO_COLOR still carries escapes: {plain}"
        );
        assert_eq!(stripped(&coloured), plain);
        assert!(plain.contains("Examples:"), "{plain}");
        assert!(plain.contains("[CONFIG]"), "{plain}");
    }

    #[test]
    fn a_refusal_reads_the_same_with_the_colour_taken_off_it() {
        let error = Cli::try_parse_from(["vigild", "--dry-run"]).expect_err("refused");

        let rendered = error.render();
        let coloured = rendered.ansi().to_string();
        let plain = rendered.to_string();

        assert!(coloured.contains('\u{1b}'));
        assert!(!plain.contains('\u{1b}'), "{plain}");
        assert_eq!(stripped(&coloured), plain);
        assert!(plain.contains("error:"), "{plain}");
    }
}
