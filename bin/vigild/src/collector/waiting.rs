use std::time::{Duration, Instant};

use vigil_collect::Health;

use crate::wizard::Surveyed;

pub const FIRST_READING: Duration = Duration::from_secs(45);

pub const LOOK_AGAIN: Duration = Duration::from_millis(500);

pub fn first_reading(
    mut asked: impl FnMut() -> Result<Surveyed, String>,
    within: Duration,
    between: Duration,
) -> Result<Surveyed, String> {
    let started = Instant::now();
    loop {
        let standing = asked()?;
        let waiting = matches!(standing.health, Health::Unavailable(_));
        if !waiting || started.elapsed() + between > within {
            return Ok(standing);
        }
        std::thread::sleep(between);
    }
}
