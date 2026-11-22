use super::health::Health;
use super::limit::Limit;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Standing {
    pub health: Health,
    pub limit: Option<Limit>,
}

impl Standing {
    pub fn of(health: Health) -> Standing {
        Standing {
            health,
            limit: None,
        }
    }

    pub fn limited(health: Health, limit: Option<Limit>) -> Standing {
        Standing { health, limit }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_standing_told_only_its_health_is_not_limited() {
        let standing = Standing::of(Health::Degraded("one owner unresolved".into()));

        assert_eq!(standing.limit, None);
        assert_eq!(
            standing.health,
            Health::Degraded("one owner unresolved".into())
        );
    }
}
