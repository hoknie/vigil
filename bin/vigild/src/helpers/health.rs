use vigil_collect::Health;

pub fn describe(health: &Health) -> String {
    match health {
        Health::Ok => "ok".to_string(),
        Health::Degraded(why) => format!("degraded:{why}"),
        Health::Unavailable(why) => format!("unavailable:{why}"),
    }
}
