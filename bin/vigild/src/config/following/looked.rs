use vigil_module::Settings;

#[derive(Debug, Default)]
pub struct Looked {
    pub said: Vec<String>,
    pub refollowed: Vec<(&'static str, Settings)>,
}
