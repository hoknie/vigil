use vigil_store::Store;

use crate::loops::Watch;

pub fn forget(store: &dyn Store, switched_off: &[String]) {
    for name in switched_off {
        match store.forget_baseline(name) {
            Ok(true) => eprintln!("  baseline {name}: dropped, the collector is off"),
            Ok(false) => {}
            Err(error) => eprintln!("  baseline {name}: could not be dropped ({error})"),
        }
    }
}

pub fn restore(store: &dyn Store, watches: &mut [Watch]) {
    for watch in watches {
        match store.baseline(watch.name()) {
            Ok(Some(baseline)) => {
                eprintln!(
                    "  baseline {}: continuing from the reading of {}",
                    watch.name(),
                    baseline.taken_at
                );
                watch.restore(baseline);
            }
            Ok(None) => eprintln!(
                "  baseline {}: none yet, the first reading becomes one",
                watch.name()
            ),
            Err(error) => eprintln!(
                "  baseline {}: unreadable ({error}), the first reading becomes one",
                watch.name()
            ),
        }
    }
}
