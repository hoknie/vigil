const ENTRY_BYTES: usize = 8;

const SOCKET: u32 = 2;

pub fn socket_descriptors(bytes: &[u8]) -> Option<Vec<i32>> {
    if !bytes.len().is_multiple_of(ENTRY_BYTES) {
        return None;
    }

    Some(
        bytes
            .as_chunks::<ENTRY_BYTES>()
            .0
            .iter()
            .filter(|entry| u32::from_ne_bytes([entry[4], entry[5], entry[6], entry[7]]) == SOCKET)
            .map(|entry| i32::from_ne_bytes([entry[0], entry[1], entry[2], entry[3]]))
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(descriptor: i32, kind: u32) -> Vec<u8> {
        let mut bytes = descriptor.to_ne_bytes().to_vec();
        bytes.extend(kind.to_ne_bytes());
        bytes
    }

    #[test]
    fn only_the_descriptors_that_are_sockets_are_kept() {
        let mut bytes = entry(0, 1);
        bytes.extend(entry(3, 2));
        bytes.extend(entry(4, 5));
        bytes.extend(entry(7, 2));

        assert_eq!(socket_descriptors(&bytes), Some(vec![3, 7]));
    }

    #[test]
    fn a_list_cut_mid_entry_is_refused_rather_than_read_as_fewer_sockets() {
        let mut bytes = entry(3, 2);
        bytes.extend(&entry(7, 2)[..5]);

        assert_eq!(socket_descriptors(&bytes), None);
        assert_eq!(socket_descriptors(&[]), Some(Vec::new()));
    }
}
