use vigil_model::{AccountChange, AccountObject, Changing, Snapshot};
use vigil_view::{Form, RowKey};

use super::{group, key, session, ssh_user, sudo, user};
use crate::helpers::NOTHING_CHANGED;
use crate::types::Subject;

pub fn form(
    subject: Subject,
    reading: &Snapshot,
    row: Option<&RowKey>,
    changing: Changing,
) -> Result<Form, String> {
    let object = offered(subject, changing)?;
    match (object, changing) {
        (_, Changing::Delete) => Err(format!(
            "a {} is deleted without a form: the band under the list asks, and D there does it",
            object.named()
        )),
        (AccountObject::User, Changing::Update) => user::update_form(reading, row),
        (AccountObject::Group, Changing::Create) => Ok(group::create_form(reading)),
        (AccountObject::Group, Changing::Update) => group::update_form(reading, row),
        (AccountObject::Sudo, Changing::Update) => sudo::update_form(reading, row),
        (AccountObject::Key, Changing::Create) => key::create_form(reading, row),
        (AccountObject::Key, Changing::Update) => key::update_form(reading, row),
        (AccountObject::SshUser, Changing::Create) => Ok(ssh_user::create_form()),
        (AccountObject::SshUser, Changing::Update) => ssh_user::update_form(reading, row),
        _ => Err(not_offered(subject, object)),
    }
}

pub fn change(
    subject: Subject,
    reading: &Snapshot,
    row: Option<&RowKey>,
    changing: Changing,
    form: Option<&Form>,
) -> Result<AccountChange, String> {
    let object = offered(subject, changing)?;
    let change = match changing {
        Changing::Delete => match object {
            AccountObject::User => user::delete(reading, row),
            AccountObject::Group => group::delete(reading, row),
            AccountObject::Sudo => sudo::delete(reading, row),
            AccountObject::Key => key::delete(reading, row),
            AccountObject::SshUser => ssh_user::delete(reading, row),
            AccountObject::Session => session::delete(reading, row),
        },
        _ => {
            let Some(form) = form else {
                return Err(format!(
                    "to {} a {}, its form is filled in first, and no form came with this",
                    changing.as_str(),
                    object.named()
                ));
            };
            match (object, changing) {
                (AccountObject::User, Changing::Update) => user::update(reading, row, form),
                (AccountObject::Group, Changing::Create) => group::create(reading, form),
                (AccountObject::Group, Changing::Update) => group::update(reading, row, form),
                (AccountObject::Sudo, Changing::Update) => sudo::update(reading, row, form),
                (AccountObject::Key, Changing::Create) => key::create(reading, form),
                (AccountObject::Key, Changing::Update) => key::update(reading, row, form),
                (AccountObject::SshUser, Changing::Create) => ssh_user::create(reading, form),
                (AccountObject::SshUser, Changing::Update) => ssh_user::update(reading, row, form),
                _ => Err(not_offered(subject, object)),
            }
        }
    }?;

    match change.is_empty() {
        true => Err(NOTHING_CHANGED.to_string()),
        false => Ok(change),
    }
}

fn offered(subject: Subject, changing: Changing) -> Result<AccountObject, String> {
    let Some(object) = subject.object() else {
        return Err(format!(
            "the {} list is read, not changed: it holds objects of a kind this console does \
             not know",
            subject.name()
        ));
    };
    match object.offers(changing) {
        true => Ok(object),
        false => Err(format!(
            "the {} list does not {} rows: it offers {}",
            subject.name(),
            changing.as_str(),
            offers(object)
        )),
    }
}

fn not_offered(subject: Subject, object: AccountObject) -> String {
    format!("the {} list offers {}", subject.name(), offers(object))
}

fn offers(object: AccountObject) -> String {
    object
        .changings()
        .iter()
        .map(|changing| changing.as_str())
        .collect::<Vec<&str>>()
        .join(", ")
}
