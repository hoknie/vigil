use serde_json::Value;
use vigil_view::Piece;

pub(super) fn table(key: &str, item: &Value) -> Vec<Piece> {
    let family = item["family"].as_str().unwrap_or("?");
    let name = item["name"].as_str().unwrap_or("?");
    let mut said = vec![
        Piece::title("TABLE", format!("{family} {name}")),
        Piece::Blank,
    ];

    for (label, value) in [
        ("family", family.to_string()),
        ("name", name.to_string()),
        ("chains", number(item, "chains")),
        ("rules", number(item, "rules")),
        ("object", key.to_string()),
    ] {
        said.push(Piece::field(label, value));
    }
    said.push(Piece::Blank);

    said.push(Piece::heading("WHAT THIS MEANS"));
    said.push(Piece::text(
        "A table holds chains, and the chains hold the rules. The number here is every rule \
         in the table, counted; the rules themselves are not read, because fail2ban and \
         docker rewrite theirs continually and a row per rule would be a finding on every \
         one of those.",
    ));
    said.push(Piece::Blank);

    said
}

fn number(item: &Value, field: &str) -> String {
    item[field]
        .as_u64()
        .map(|count| count.to_string())
        .unwrap_or_else(|| "—".to_string())
}
