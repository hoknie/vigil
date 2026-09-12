use vigil_collect::Health;

pub fn in_words(described: &str) -> String {
    match described.split_once(':') {
        Some((state_of_it, why)) => format!("{state_of_it} — {why}"),
        None => described.to_string(),
    }
}

pub fn describe(health: &Health) -> String {
    match health {
        Health::Ok => "ok".to_string(),
        Health::Degraded(why) => format!("degraded:{why}"),
        Health::Unavailable(why) => format!("unavailable:{why}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_a_collector_was_is_put_to_a_reader_in_the_words_the_log_uses() {
        let was = describe(&Health::Degraded("auditd has brought nothing".into()));

        assert_eq!(in_words(&was), "degraded — auditd has brought nothing");
        assert_eq!(
            in_words(&describe(&Health::Ok)),
            "ok",
            "a collector that was fine carries no reason, and no dangling dash either"
        );
    }
}
