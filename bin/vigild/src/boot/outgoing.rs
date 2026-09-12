use std::collections::BTreeMap;
use std::path::Path;

use vigil_report::Reporter;
use vigil_store::{Limits, Outgoing};

pub fn open(state_dir: &Path, reporters: &[Box<dyn Reporter>]) -> Result<Vec<Outgoing>, String> {
    let directory = state_dir.join("outgoing");
    let mut taken: BTreeMap<String, usize> = BTreeMap::new();
    let mut buffers = Vec::new();

    for reporter in reporters {
        let name = told_apart(reporter.name(), &mut taken);
        let path = directory.join(format!("{name}.ndjson"));
        let buffer = Outgoing::open(name, path, Limits::outgoing())
            .map_err(|error| format!("the outgoing buffer for {}: {error}", reporter.name()))?;
        buffers.push(buffer);
    }

    Ok(buffers)
}

fn told_apart(name: &str, taken: &mut BTreeMap<String, usize>) -> String {
    let seen = taken.entry(name.to_string()).or_insert(0);
    *seen += 1;

    match *seen {
        1 => name.to_string(),
        nth => format!("{name}-{nth}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_receivers_of_the_same_kind_do_not_share_one_file() {
        let mut taken = BTreeMap::new();

        let first = told_apart("ndjson", &mut taken);
        let second = told_apart("ndjson", &mut taken);
        let other = told_apart("syslog", &mut taken);

        assert_eq!(first, "ndjson");
        assert_ne!(
            first, second,
            "one file for two receivers replays to whichever answered first"
        );
        assert_eq!(other, "syslog");
    }
}
