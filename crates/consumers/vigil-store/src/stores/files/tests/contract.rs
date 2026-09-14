use super::harness::temporary_directory;
use crate::conformance;
use crate::stores::files::FileStore;

#[test]
fn it_satisfies_the_contract_every_backend_has_to_satisfy() {
    conformance::run_all(&|| {
        Box::new(FileStore::open(&temporary_directory("conformance")).expect("opens"))
    });
}
