#[cfg(test)]
mod tests;

mod cells;
mod columns;
mod facts;
mod fields;
mod kind;
mod notices;
mod regions;
mod render;
mod row;
mod rows;
mod showing;
mod tally;

pub use facts::{
    answers, attended, could_log_in, groups_of, password, privileged, readable, route_to_root,
    seen_by, sudo_for, what,
};
pub use fields::{flag, members, number, objects, rules, text};
pub use kind::Kind;
pub use render::{printed_height, render};
pub use row::Row;
pub use rows::{keys, rows};
pub use showing::Showing;
