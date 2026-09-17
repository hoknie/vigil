use serde_json::Value;
use vigil_module::Module;

pub struct Follower {
    pub module: Box<dyn Module>,
    pub key: &'static str,
    pub applied: Value,
}
