use std::cell::Cell;
use std::time::Duration;

use vigil_collect::Health;

use crate::collector::waiting::{FIRST_READING, first_reading};
use crate::wizard::Surveyed;

fn standing(health: Health) -> Surveyed {
    Surveyed {
        name: "firewall".into(),
        health,
    }
}

#[test]
fn a_reading_the_job_just_started_has_not_written_yet_is_waited_for_rather_than_refused() {
    let asked = Cell::new(0);

    let found = first_reading(
        || {
            asked.set(asked.get() + 1);
            Ok(standing(match asked.get() {
                1..=3 => Health::Unavailable("the dump is not there yet".into()),
                _ => Health::Ok,
            }))
        },
        Duration::from_secs(5),
        Duration::from_millis(1),
    )
    .expect("answers");

    assert_eq!(found.health, Health::Ok);
    assert_eq!(
        asked.get(),
        4,
        "the job was started a moment ago: a reading it has not written yet is not a host \
         that cannot be read, and refusing it left firewall switched off on every fresh install"
    );
}

#[test]
fn a_reading_that_never_comes_is_refused_once_the_wait_is_over_and_not_waited_for_for_ever() {
    let found = first_reading(
        || Ok(standing(Health::Unavailable("nft is not installed".into()))),
        Duration::from_millis(20),
        Duration::from_millis(5),
    )
    .expect("answers");

    assert!(matches!(found.health, Health::Unavailable(_)));
}

#[test]
fn a_reading_that_is_there_at_once_is_not_waited_for_at_all() {
    let asked = Cell::new(0);

    first_reading(
        || {
            asked.set(asked.get() + 1);
            Ok(standing(Health::Degraded("less than all of it".into())))
        },
        Duration::from_secs(5),
        Duration::from_secs(5),
    )
    .expect("answers");

    assert_eq!(asked.get(), 1);
}

#[test]
fn the_slowest_first_dump_this_product_writes_fits_in_the_wait() {
    assert!(
        FIRST_READING >= Duration::from_secs(30),
        "the container engines' dump took six seconds on a quiet Mac and its clients have a \
         deadline of their own; an installer that gives up sooner reports a working host as \
         one that cannot be read"
    );
}
