use std::time::{Duration, Instant};

use vigil_view::Piece;

use super::{DRAWN, Graph, KEPT};

const A_SECOND: u64 = 1;

const LEVELS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

const WAITING: &str = "waiting for a second reading";

const OFF: &str = "Counting is off. Press w to follow what goes through this interface: the \
                   console is already reading this host every two seconds while this screen \
                   is open, so nothing more is asked of the agent — the numbers are the ones \
                   that reading carries. Counting stops the moment you leave this panel.";

const NOT_COUNTED: &str = "This host is not counting what goes through its interfaces, so \
                           there is no number to follow. Set monitoring under firewall in \
                           vigil.yaml and restart the agent: it is off by default because a \
                           packet counter moves on every reading, and a reading that moves on \
                           every pass is a line in the journal once a minute for the life of \
                           the host.";

impl Graph {
    pub fn pieces(&self, at: Instant, counted: Option<u64>) -> Vec<Piece> {
        let mut said = self.drawn.clone();
        said.push(Piece::heading("MONITORING"));

        match (self.watching, counted) {
            (false, _) => said.push(Piece::text(OFF)),
            (true, None) => said.push(Piece::warning(NOT_COUNTED)),
            (true, Some(counted)) => said.extend(self.followed(at, counted)),
        }
        said.push(Piece::Blank);

        said
    }

    fn followed(&self, at: Instant, counted: u64) -> Vec<Piece> {
        let watched = self.since.map(|since| at.saturating_duration_since(since));

        let mut said = vec![
            Piece::Blank,
            Piece::field(
                "counted",
                format!("{counted} packet(s) since this host booted"),
            ),
            Piece::field(
                "watching",
                watched.map_or("just started".to_string(), said_for),
            ),
            Piece::field("readings", format!("{} of {KEPT} kept", self.samples.len())),
            Piece::field("rate", self.rate(watched)),
            Piece::Blank,
        ];
        if let Some(bar) = self.bar() {
            said.push(Piece::line(format!("   {bar}")));
            said.push(Piece::Blank);
        }

        said
    }

    fn moved(&self) -> Option<u64> {
        let first = self.samples.front()?;
        let last = self.samples.back()?;

        Some(last.saturating_sub(*first))
    }

    fn rate(&self, watched: Option<Duration>) -> String {
        let (Some(moved), Some(watched)) = (self.moved(), watched) else {
            return WAITING.to_string();
        };
        if watched.as_secs() < A_SECOND || self.samples.len() < 2 {
            return WAITING.to_string();
        }

        format!(
            "{moved} packet(s) over {}, {:.1} a second",
            said_for(watched),
            moved as f64 / watched.as_secs_f64()
        )
    }

    pub(super) fn bar(&self) -> Option<String> {
        let steps: Vec<u64> = self
            .samples
            .iter()
            .zip(self.samples.iter().skip(1))
            .map(|(before, after)| after.saturating_sub(*before))
            .collect();
        let widest = *steps.iter().max()?;
        if widest == 0 {
            return None;
        }

        let drawn: Vec<&u64> = steps.iter().rev().take(DRAWN).collect();
        Some(
            drawn
                .into_iter()
                .rev()
                .map(|step| LEVELS[((step * (LEVELS.len() as u64 - 1)) / widest) as usize])
                .collect(),
        )
    }
}

fn said_for(watched: Duration) -> String {
    match watched.as_secs() {
        0 => "under a second".to_string(),
        one => format!("{one} second(s)"),
    }
}
