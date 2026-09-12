#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelModule {
    pub name: String,
    pub size: u64,
    pub dependencies: Vec<String>,
    pub state: String,
}

pub fn parse_modules(text: &str) -> Vec<KernelModule> {
    let mut modules = Vec::new();

    for line in text.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 5 {
            continue;
        }
        let Ok(size) = fields[1].parse::<u64>() else {
            continue;
        };

        modules.push(KernelModule {
            name: fields[0].to_string(),
            size,
            dependencies: match fields[3] {
                "-" => Vec::new(),
                list => list
                    .split(',')
                    .filter(|name| !name.is_empty())
                    .map(str::to_string)
                    .collect(),
            },
            state: fields[4].to_string(),
        });
    }

    modules
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_module_with_and_without_dependants() {
        let modules = parse_modules(
            "nf_conntrack 176128 6 xt_conntrack,nf_nat,xt_MASQUERADE, Live 0xffffffffc0a12000\n\
             loop 32768 0 - Live 0x0000000000000000\n",
        );

        assert_eq!(modules.len(), 2);
        assert_eq!(modules[0].name, "nf_conntrack");
        assert_eq!(modules[0].size, 176128);
        assert_eq!(
            modules[0].dependencies,
            vec!["xt_conntrack", "nf_nat", "xt_MASQUERADE"]
        );
        assert_eq!(modules[0].state, "Live");
        assert!(modules[1].dependencies.is_empty());
    }

    #[test]
    fn the_reference_count_is_read_past_rather_than_recorded() {
        let first = parse_modules("nf_conntrack 176128 6 - Live 0x0\n");
        let later = parse_modules("nf_conntrack 176128 41 - Live 0x0\n");

        assert_eq!(first, later);
    }

    #[test]
    fn a_line_in_an_unknown_shape_is_skipped_rather_than_half_read() {
        assert!(parse_modules("nf_conntrack 176128\n").is_empty());
        assert!(parse_modules("nf_conntrack not-a-size 0 - Live 0x0\n").is_empty());
    }
}
