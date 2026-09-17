use vigil_model::{AccountObject, Changing};

pub(super) fn not_offered(object: AccountObject, changing: Changing) -> String {
    let done = match changing {
        Changing::Create => "created",
        Changing::Update => "edited",
        Changing::Delete => "deleted",
    };
    sentence(&format!(
        "{}s are not {done} from this console",
        object.named()
    ))
}

pub(in crate::ui::app::deeds) fn sentence(said: &str) -> String {
    let said = said.trim();
    let mut characters = said.chars();
    let mut out: String = match characters.next() {
        Some(first) => first.to_uppercase().chain(characters).collect(),
        None => return String::new(),
    };
    if !out.ends_with(['.', '!', '?']) {
        out.push('.');
    }
    out
}
