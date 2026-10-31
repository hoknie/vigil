#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessorCounters {
    pub processors: u64,
    pub total: u64,
    pub busy: u64,
}

const COUNTED_FIELDS: usize = 8;

const IDLE: usize = 3;

const IOWAIT: usize = 4;

pub fn parse_proc_stat(text: &str) -> Option<ProcessorCounters> {
    let mut summary = None;
    let mut processors = 0;

    for line in text.lines() {
        let mut words = line.split_whitespace();
        let Some(name) = words.next() else {
            continue;
        };
        if name == "cpu" {
            let fields: Vec<u64> = words
                .take(COUNTED_FIELDS)
                .map(str::parse)
                .collect::<Result<_, _>>()
                .ok()?;
            if fields.len() <= IDLE {
                return None;
            }
            let total: u64 = fields.iter().sum();
            let idle = fields[IDLE] + fields.get(IOWAIT).copied().unwrap_or(0);
            summary = Some((total, total.saturating_sub(idle)));
        } else if name
            .strip_prefix("cpu")
            .is_some_and(|rest| !rest.is_empty() && rest.bytes().all(|byte| byte.is_ascii_digit()))
        {
            processors += 1;
        }
    }

    let (total, busy) = summary?;
    match processors {
        0 => None,
        processors => Some(ProcessorCounters {
            processors,
            total,
            busy,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OF_A_FOUR_PROCESSOR_HOST: &str = "\
cpu  4705 356 584 3699 23 23 0 0 0 0
cpu0 1393 280 260 914 5 13 0 0 0 0
cpu1 1166 22 113 952 6 4 0 0 0 0
cpu2 1094 34 109 917 6 3 0 0 0 0
cpu3 1052 20 102 916 6 3 0 0 0 0
intr 114930548 113199788 3 0 5 263 0 4 [...]
ctxt 1990473
btime 1062191376
processes 2915
procs_running 1
";

    #[test]
    fn the_counters_of_all_processors_are_read_with_idle_and_iowait_as_not_busy() {
        let counters = parse_proc_stat(OF_A_FOUR_PROCESSOR_HOST).expect("reads");

        assert_eq!(counters.processors, 4);
        assert_eq!(counters.total, 4705 + 356 + 584 + 3699 + 23 + 23);
        assert_eq!(counters.busy, 4705 + 356 + 584 + 23);
    }

    #[test]
    fn a_kernel_line_without_iowait_is_still_read() {
        let old = "cpu  4705 356 584 3699\ncpu0 4705 356 584 3699\n";

        let counters = parse_proc_stat(old).expect("reads");

        assert_eq!(counters.busy, 4705 + 356 + 584);
    }

    #[test]
    fn guest_time_is_not_counted_twice() {
        let with_guest = "cpu  100 0 0 100 0 0 0 0 50 0\ncpu0 100 0 0 100 0 0 0 0 50 0\n";

        let counters = parse_proc_stat(with_guest).expect("reads");

        assert_eq!(
            counters.total, 200,
            "guest time is already inside user time, and adding it again makes a host look busier"
        );
    }

    #[test]
    fn a_file_in_a_shape_we_do_not_know_is_not_an_idle_host() {
        assert_eq!(parse_proc_stat(""), None);
        assert_eq!(parse_proc_stat("cpu  a b c d\ncpu0 1 2 3 4\n"), None);
        assert_eq!(parse_proc_stat("cpu  1 2 3 4\n"), None);
    }
}
