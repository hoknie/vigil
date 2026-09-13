use serde_json::Value;

pub struct AccountView<'a>(&'a Value);

impl<'a> AccountView<'a> {
    pub fn new(value: &'a Value) -> Self {
        AccountView(value)
    }

    pub fn name(&self) -> &str {
        self.0["name"].as_str().unwrap_or("?")
    }

    pub fn uid(&self) -> u64 {
        self.0["uid"].as_u64().unwrap_or(u64::MAX)
    }

    pub fn shell(&self) -> &str {
        self.0["shell"].as_str().unwrap_or("")
    }

    pub fn home(&self) -> &str {
        self.0["home"].as_str().unwrap_or("")
    }

    pub fn interactive(&self) -> bool {
        self.0["interactive"].as_bool().unwrap_or(false)
    }

    pub fn password(&self) -> Option<&str> {
        self.0["password"].as_str()
    }

    pub fn last_change_day(&self) -> Option<i64> {
        self.0["password_last_change_day"].as_i64()
    }

    pub fn shadow_readable(&self) -> bool {
        self.0["shadow_readable"].as_bool().unwrap_or(false)
    }

    pub fn password_permits_login(&self) -> bool {
        self.0["password_permits_login"].as_bool().unwrap_or(false)
    }

    pub fn is_superuser(&self) -> bool {
        self.uid() == 0
    }

    pub fn describe(&self) -> String {
        format!(
            "{} (uid {}, shell {})",
            self.name(),
            self.uid(),
            self.shell()
        )
    }
}
