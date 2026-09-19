const STATE: &str = "(State = ";

const TOTAL: &str = "Total number of apps";

const ALLOWED: &str = "(Allow incoming connections)";

const BLOCKED: &str = "(Block incoming connections)";

pub const MOST_APPLICATIONS: usize = 512;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ApplicationFirewall {
    pub state: Option<u8>,
    pub blocks_all: Option<bool>,
    pub stealth: Option<bool>,
    pub allows_signed: Option<bool>,
    pub allows_downloaded_signed: Option<bool>,
    pub applications: Vec<(String, bool)>,
}

impl ApplicationFirewall {
    pub fn enabled(&self) -> Option<bool> {
        self.state.map(|state| state > 0)
    }
}

pub fn parse_application_firewall(printed: &str) -> Option<ApplicationFirewall> {
    let mut read = ApplicationFirewall::default();
    let mut listed: Option<String> = None;

    for line in printed.lines().map(str::trim) {
        let lower = line.to_ascii_lowercase();

        if let Some(at) = line.find(STATE) {
            read.state = line[at + STATE.len()..]
                .split(')')
                .next()
                .and_then(|number| number.trim().parse().ok());
        } else if lower.contains("block all") {
            read.blocks_all = Some(switched_on(&lower));
        } else if lower.contains("stealth") {
            read.stealth = Some(switched_on(&lower));
        } else if lower.contains("built-in signed") {
            read.allows_signed = Some(switched_on(&lower));
        } else if lower.contains("downloaded signed") {
            read.allows_downloaded_signed = Some(switched_on(&lower));
        } else if line.starts_with(TOTAL) {
            continue;
        } else if line == ALLOWED || line == BLOCKED {
            if let Some(path) = listed.take()
                && read.applications.len() < MOST_APPLICATIONS
            {
                read.applications.push((path, line == ALLOWED));
            }
        } else if let Some((number, path)) = line.split_once(" : ")
            && number.trim().bytes().all(|byte| byte.is_ascii_digit())
            && path.trim().starts_with('/')
        {
            listed = Some(path.trim().to_string());
        }
    }

    read.state.map(|_| read)
}

fn switched_on(lower: &str) -> bool {
    lower.contains("enabled") || lower.ends_with(" on") || lower.contains(" on.")
}
