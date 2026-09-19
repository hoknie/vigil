use super::meminfo::MemoryFacts;

pub fn parse_memory_size(bytes: &[u8]) -> Option<MemoryFacts> {
    let total_bytes = u64::from_ne_bytes(bytes.get(..8)?.try_into().ok()?);

    (total_bytes > 0).then_some(MemoryFacts {
        total_bytes,
        swap_total_bytes: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_memory_of_a_mac_is_its_size_and_its_swap_is_not_a_size_it_has() {
        let memory = parse_memory_size(&25_769_803_776u64.to_ne_bytes()).expect("parses");

        assert_eq!(memory.total_bytes, 25_769_803_776);
        assert_eq!(
            memory.swap_total_bytes, None,
            "macOS grows its swap a file at a time while it runs and shrinks it again, so its \
             total would move between readings on a host where nothing was changed"
        );
    }

    #[test]
    fn a_value_cut_short_or_empty_is_no_memory() {
        assert_eq!(parse_memory_size(&[1, 2, 3]), None);
        assert_eq!(parse_memory_size(&0u64.to_ne_bytes()), None);
    }
}
