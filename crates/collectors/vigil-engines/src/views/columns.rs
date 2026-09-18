use vigil_view::{Column, Room, Width};

use crate::types::List;

const ROOM_FOR_MORE: u16 = 118;

pub(super) struct Shown {
    pub header: &'static str,
    pub width: Width,
    pub narrow: bool,
    pub counted: bool,
}

const fn shown(header: &'static str, width: Width, narrow: bool, counted: bool) -> Shown {
    Shown {
        header,
        width,
        narrow,
        counted,
    }
}

const CONTAINERS: &[Shown] = &[
    shown("NAME", Width::Least(16), true, false),
    shown("PROJECT", Width::Fixed(10), false, false),
    shown("IMAGE", Width::Share(2), true, false),
    shown("NETWORKS", Width::Share(1), true, false),
    shown("PORTS", Width::Share(1), false, false),
    shown("HOST PATHS", Width::Fixed(10), true, true),
];

const IMAGES: &[Shown] = &[
    shown("TAGS", Width::Share(2), true, false),
    shown("ID", Width::Fixed(19), true, false),
    shown("DIGEST", Width::Share(1), false, false),
    shown("SIZE", Width::Fixed(9), true, false),
    shown("IN USE", Width::Fixed(6), true, true),
];

const VOLUMES: &[Shown] = &[
    shown("NAME", Width::Least(16), true, false),
    shown("PROJECT", Width::Fixed(10), false, false),
    shown("DRIVER", Width::Fixed(8), true, false),
    shown("BINDS", Width::Share(1), true, false),
    shown("MOUNTPOINT", Width::Share(2), false, false),
];

const NETWORKS: &[Shown] = &[
    shown("NAME", Width::Least(14), true, false),
    shown("PROJECT", Width::Fixed(10), false, false),
    shown("DRIVER", Width::Fixed(8), true, false),
    shown("SUBNETS", Width::Share(1), true, false),
    shown("INTERNAL", Width::Fixed(8), true, false),
    shown("INTERFACE", Width::Fixed(10), false, false),
];

const COMPOSE: &[Shown] = &[
    shown("PROJECT", Width::Least(12), true, false),
    shown("SERVICES", Width::Share(2), true, false),
    shown("CONTAINERS", Width::Fixed(10), true, true),
    shown("DIRECTORY", Width::Share(2), false, false),
];

const PODS: &[Shown] = &[
    shown("POD", Width::Least(12), true, false),
    shown("CONTAINERS", Width::Share(2), true, false),
    shown("NETWORKS", Width::Share(1), true, false),
    shown("CGROUP", Width::Fixed(14), false, false),
];

const SECRETS: &[Shown] = &[
    shown("SECRET", Width::Least(16), true, false),
    shown("DRIVER", Width::Fixed(8), true, false),
    shown("CREATED", Width::Fixed(20), false, false),
    shown("UPDATED", Width::Fixed(20), true, false),
];

const REGISTRIES: &[Shown] = &[
    shown("REGISTRY", Width::Least(20), true, false),
    shown("TLS", Width::Fixed(8), true, false),
    shown("ROLE", Width::Fixed(10), true, false),
    shown("FROM THE FILE", Width::Share(1), false, false),
];

pub(super) fn every(list: List) -> &'static [Shown] {
    match list {
        List::Containers => CONTAINERS,
        List::Images => IMAGES,
        List::Volumes => VOLUMES,
        List::Networks => NETWORKS,
        List::Compose => COMPOSE,
        List::Pods => PODS,
        List::Secrets => SECRETS,
        List::Registries => REGISTRIES,
    }
}

pub(super) fn drawn(list: List, room: Room) -> Vec<usize> {
    let wide = room.holds(ROOM_FOR_MORE);
    every(list)
        .iter()
        .enumerate()
        .filter(|(_, column)| wide || column.narrow)
        .map(|(at, _)| at)
        .collect()
}

pub(super) fn columns(list: List, room: Room) -> Vec<Column> {
    let every = every(list);
    drawn(list, room)
        .into_iter()
        .map(|at| Column::new(every[at].header, every[at].width))
        .collect()
}

pub(super) fn sorted_by(list: List) -> Vec<&'static str> {
    every(list).iter().map(|column| column.header).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_list_draws_fewer_columns_in_eighty_than_in_a_wide_terminal_and_sorts_by_all() {
        for list in List::ALL {
            let narrow = columns(list, Room::of(80)).len();
            let wide = columns(list, Room::of(160)).len();

            assert!(
                narrow >= 3 && narrow < wide,
                "{}: {narrow} and {wide}",
                list.name()
            );
            assert_eq!(sorted_by(list).len(), wide, "{}", list.name());
        }
    }
}
