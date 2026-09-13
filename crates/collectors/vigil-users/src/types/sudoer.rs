use serde_json::Value;

pub struct SudoerView<'a>(&'a Value);

impl<'a> SudoerView<'a> {
    pub fn new(value: &'a Value) -> Self {
        SudoerView(value)
    }

    pub fn who(&self) -> &str {
        self.0["who"].as_str().unwrap_or("?")
    }

    pub fn is_group(&self) -> bool {
        self.0["group"].as_bool().unwrap_or(false)
    }

    pub fn nopasswd(&self) -> bool {
        self.0["nopasswd"].as_bool().unwrap_or(false)
    }

    pub fn all_commands(&self) -> bool {
        self.0["all_commands"].as_bool().unwrap_or(false)
    }

    pub fn spec_redacted(&self) -> bool {
        self.0["spec_redacted"].as_bool().unwrap_or(false)
    }

    pub fn sources(&self) -> Vec<&str> {
        self.rules()
            .into_iter()
            .filter_map(|rule| rule["source"].as_str())
            .collect()
    }

    pub fn specs(&self) -> Vec<&str> {
        self.rules()
            .into_iter()
            .filter_map(|rule| rule["spec"].as_str())
            .collect()
    }

    fn rules(&self) -> Vec<&'a Value> {
        self.0["rules"]
            .as_array()
            .map(|rules| rules.iter().collect())
            .unwrap_or_default()
    }

    pub fn is_unrestricted(&self) -> bool {
        self.nopasswd() && self.all_commands()
    }
}
