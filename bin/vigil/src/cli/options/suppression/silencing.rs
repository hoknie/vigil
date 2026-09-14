use clap::Subcommand;

use super::naming::Naming;
use super::reading::Reading;
use super::silence::Silence;

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Silencing {
    #[command(about = "Stop reporting the object a finding is about")]
    Add(Silence),

    #[command(about = "Report it again: take the entry out of the configuration")]
    Remove(Naming),

    #[command(about = "What the configuration silences now")]
    List(Reading),
}
