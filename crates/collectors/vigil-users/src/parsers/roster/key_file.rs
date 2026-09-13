use crate::parsers::authorized_keys::AuthorizedKey;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserKeyFile {
    pub user: String,
    pub uid: u32,
    pub path: String,
    pub readable: bool,
    pub keys: Vec<AuthorizedKey>,
}
