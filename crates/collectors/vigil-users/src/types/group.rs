use serde_json::Value;

pub struct GroupView<'a>(&'a Value);

impl<'a> GroupView<'a> {
    pub fn new(value: &'a Value) -> Self {
        GroupView(value)
    }

    pub fn name(&self) -> &str {
        self.0["name"].as_str().unwrap_or("?")
    }

    pub fn privileged(&self) -> bool {
        self.0["privileged"].as_bool().unwrap_or(false)
    }

    pub fn privilege(&self) -> Option<&str> {
        self.0["privilege"].as_str()
    }

    pub fn members(&self) -> Vec<&str> {
        self.0["members"]
            .as_array()
            .map(|members| members.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default()
    }

    pub fn members_gained_since(&self, was: &GroupView<'_>) -> Vec<&'a str> {
        let before = was.members();
        self.0["members"]
            .as_array()
            .map(|members| {
                members
                    .iter()
                    .filter_map(Value::as_str)
                    .filter(|member| !before.contains(member))
                    .collect()
            })
            .unwrap_or_default()
    }
}
