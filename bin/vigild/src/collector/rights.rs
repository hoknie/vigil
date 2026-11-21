use super::manager::Manager;

pub fn refused_without_root(per_account: bool, name: &str, unit: Option<&str>) -> Option<String> {
    let unit = unit.filter(|_| per_account)?;
    Some(format!(
        "{name} is read from what the launchd job {} writes, and that job runs as root: run this with sudo",
        Manager::Launchd.job(unit)
    ))
}

pub fn only_with_the_rights_it_needs(name: &str) -> Result<(), String> {
    let per_account = crate::wizard::per_account();
    match refused_without_root(per_account, name, crate::modules::unit_of(name)) {
        Some(refusal) => Err(refusal),
        None => Ok(()),
    }
}
