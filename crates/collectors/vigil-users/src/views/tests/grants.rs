use super::host::a_host_with_names_that_prefix_one_another as host;
use super::walks;
use crate::fixture::users;
use crate::types::Kind;
use crate::views::facts::{grant_to_group, reached_by, sudo_for};
use crate::views::fields::text;

#[test]
fn the_grants_that_reach_an_account_are_found_by_key_and_through_its_groups_as_the_walk_found_them()
{
    for reading in [host(), users()] {
        for (key, account) in walks::every(&reading, Kind::Account) {
            let name = text(account, "name").unwrap_or_default();
            let found: Vec<&str> = sudo_for(&reading, account)
                .into_iter()
                .map(|(key, _)| key)
                .collect();
            let walked: Vec<&str> = walks::sudo_for(&reading, name)
                .into_iter()
                .map(|(key, _)| key)
                .collect();

            assert_eq!(
                found, walked,
                "{key}: a grant to a group is looked up under each group the account lists, and \
                 it must reach the same accounts a walk over every group reached"
            );
        }
    }
}

#[test]
fn the_grant_to_a_group_is_the_one_key_it_can_live_under_and_the_walk_over_every_grant_found_no_other()
 {
    let reading = host();
    let mut groups: Vec<&str> = walks::every(&reading, Kind::Group)
        .into_iter()
        .filter_map(|(_, group)| text(group, "name"))
        .collect();
    groups.extend(["ghost", "nobody", "dock"]);

    for group in groups {
        assert_eq!(
            grant_to_group(&reading, group)
                .into_iter()
                .collect::<Vec<_>>(),
            walks::grants_to_group(&reading, group),
            "{group}: the detail of a group reads its grant from sudoer|%{group} instead of \
             walking every grant, and must find the one the walk found"
        );
    }
}

#[test]
fn the_accounts_a_grant_reaches_are_looked_up_by_name_in_the_order_the_walk_over_every_account_gave()
 {
    for reading in [host(), users()] {
        let mut principals: Vec<&str> = walks::every(&reading, Kind::Sudoer)
            .into_iter()
            .filter_map(|(_, grant)| text(grant, "who"))
            .collect();
        principals.extend(["%ghost", "%users", "nobody", "deploy", "%"]);

        for who in principals {
            assert_eq!(
                reached_by(&reading, who),
                walks::reached_by(&reading, who),
                "{who}: the accounts a grant reaches are the members of its group that are \
                 accounts, looked up by name, and a reader must see them in the order the walk \
                 over every account listed them"
            );
        }
    }
}
