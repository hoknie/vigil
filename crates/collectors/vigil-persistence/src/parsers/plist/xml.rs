use std::collections::BTreeMap;

use super::property_list::DEEPEST;
use super::refusal::PlistRefusal;
use super::value::PlistValue;

#[derive(Debug, PartialEq, Eq)]
enum Tag<'a> {
    Open(&'a str),
    Close(&'a str),
    Empty(&'a str),
}

struct Reader<'a> {
    text: &'a str,
    at: usize,
}

pub(super) fn parse(bytes: &[u8]) -> Result<PlistValue, PlistRefusal> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| PlistRefusal::Broken("the XML is not UTF-8".to_string()))?;
    let text = text.trim_start_matches('\u{feff}');
    if text.trim().is_empty() {
        return Err(PlistRefusal::Empty);
    }

    let mut reader = Reader { text, at: 0 };
    match reader.tag() {
        Ok(Tag::Open("plist")) => {}
        _ => return Err(PlistRefusal::NotAPropertyList),
    }
    let tag = reader.tag()?;
    let value = reader.value(tag, 0)?;
    match reader.tag()? {
        Tag::Close("plist") => Ok(value),
        other => Err(broken(format!(
            "{other:?} after the value, where </plist> was due"
        ))),
    }
}

fn broken(why: impl Into<String>) -> PlistRefusal {
    PlistRefusal::Broken(why.into())
}

impl<'a> Reader<'a> {
    fn rest(&self) -> &'a str {
        &self.text[self.at..]
    }

    fn skip(&mut self, opened: &str, closed: &str) -> Result<(), PlistRefusal> {
        let from = self.at + opened.len();
        let end = self.text[from..]
            .find(closed)
            .ok_or_else(|| broken(format!("{opened} is never closed")))?;
        self.at = from + end + closed.len();
        Ok(())
    }

    fn tag(&mut self) -> Result<Tag<'a>, PlistRefusal> {
        loop {
            let skipped = self.rest().len() - self.rest().trim_start().len();
            self.at += skipped;
            let rest = self.rest();
            if rest.is_empty() {
                return Err(broken("the document ends inside the property list"));
            }
            if !rest.starts_with('<') {
                return Err(broken(format!(
                    "text where a tag was due: {:?}",
                    rest.chars().take(24).collect::<String>()
                )));
            }
            if rest.starts_with("<?") {
                self.skip("<?", "?>")?;
                continue;
            }
            if rest.starts_with("<!--") {
                self.skip("<!--", "-->")?;
                continue;
            }
            if rest.starts_with("<!") {
                self.skip("<!", ">")?;
                continue;
            }

            let end = rest
                .find('>')
                .ok_or_else(|| broken("a tag is never closed"))?;
            let inside = &rest[1..end];
            self.at += end + 1;

            if let Some(name) = inside.strip_prefix('/') {
                return Ok(Tag::Close(name.trim()));
            }
            if let Some(name) = inside.strip_suffix('/') {
                return Ok(Tag::Empty(
                    name.split_whitespace().next().unwrap_or_default(),
                ));
            }
            return Ok(Tag::Open(
                inside.split_whitespace().next().unwrap_or_default(),
            ));
        }
    }

    fn text(&mut self, name: &str) -> Result<String, PlistRefusal> {
        let closing = format!("</{name}>");
        let mut said = String::new();
        loop {
            let rest = self.rest();
            if let Some(cdata) = rest.strip_prefix("<![CDATA[") {
                let end = cdata
                    .find("]]>")
                    .ok_or_else(|| broken("a CDATA section is never closed"))?;
                said.push_str(&cdata[..end]);
                self.at += "<![CDATA[".len() + end + "]]>".len();
                continue;
            }
            if rest.starts_with(&closing) {
                self.at += closing.len();
                return Ok(said);
            }
            let next = rest
                .find('<')
                .ok_or_else(|| broken(format!("<{name}> is never closed")))?;
            if next == 0 {
                return Err(broken(format!("a tag inside <{name}>")));
            }
            said.push_str(&decoded(&rest[..next])?);
            self.at += next;
        }
    }

    fn value(&mut self, tag: Tag<'a>, depth: usize) -> Result<PlistValue, PlistRefusal> {
        if depth > DEEPEST {
            return Err(PlistRefusal::TooDeep);
        }
        match tag {
            Tag::Empty("dict") => Ok(PlistValue::Dictionary(BTreeMap::new())),
            Tag::Empty("array") => Ok(PlistValue::Array(Vec::new())),
            Tag::Empty("string") => Ok(PlistValue::String(String::new())),
            Tag::Empty("true") => Ok(PlistValue::Boolean(true)),
            Tag::Empty("false") => Ok(PlistValue::Boolean(false)),
            Tag::Empty("data") => Ok(PlistValue::Data(0)),
            Tag::Open("true") => self.closed("true").map(|_| PlistValue::Boolean(true)),
            Tag::Open("false") => self.closed("false").map(|_| PlistValue::Boolean(false)),
            Tag::Open("string") => self.text("string").map(PlistValue::String),
            Tag::Open("date") => self.text("date").map(PlistValue::Date),
            Tag::Open("data") => self.text("data").map(|written| {
                let letters = written
                    .bytes()
                    .filter(|byte| !byte.is_ascii_whitespace() && *byte != b'=')
                    .count();
                PlistValue::Data(letters * 3 / 4)
            }),
            Tag::Open("integer") => {
                let written = self.text("integer")?;
                integer(written.trim())
                    .map(PlistValue::Integer)
                    .ok_or_else(|| broken(format!("{written:?} is not an integer")))
            }
            Tag::Open("real") => {
                let written = self.text("real")?;
                written
                    .trim()
                    .parse()
                    .map(PlistValue::Real)
                    .map_err(|_| broken(format!("{written:?} is not a number")))
            }
            Tag::Open("array") => {
                let mut values = Vec::new();
                loop {
                    match self.tag()? {
                        Tag::Close("array") => return Ok(PlistValue::Array(values)),
                        tag => values.push(self.value(tag, depth + 1)?),
                    }
                }
            }
            Tag::Open("dict") => {
                let mut entries = BTreeMap::new();
                loop {
                    let key = match self.tag()? {
                        Tag::Close("dict") => return Ok(PlistValue::Dictionary(entries)),
                        Tag::Open("key") => self.text("key")?,
                        Tag::Empty("key") => String::new(),
                        other => return Err(broken(format!("{other:?} where a key was due"))),
                    };
                    let tag = self.tag()?;
                    let value = self.value(tag, depth + 1)?;
                    entries.insert(key, value);
                }
            }
            other => Err(broken(format!("{other:?} is no value of a property list"))),
        }
    }

    fn closed(&mut self, name: &str) -> Result<(), PlistRefusal> {
        match self.tag()? {
            Tag::Close(closing) if closing == name => Ok(()),
            other => Err(broken(format!("{other:?} where </{name}> was due"))),
        }
    }
}

fn integer(written: &str) -> Option<i128> {
    match written
        .strip_prefix("0x")
        .or_else(|| written.strip_prefix("0X"))
    {
        Some(hex) => i128::from_str_radix(hex, 16).ok(),
        None => written.parse().ok(),
    }
}

fn decoded(raw: &str) -> Result<String, PlistRefusal> {
    let mut said = String::with_capacity(raw.len());
    let mut rest = raw;
    while let Some(at) = rest.find('&') {
        said.push_str(&rest[..at]);
        let after = &rest[at + 1..];
        let end = after
            .find(';')
            .ok_or_else(|| broken("an entity is never closed"))?;
        let entity = &after[..end];
        let character = match entity {
            "lt" => '<',
            "gt" => '>',
            "amp" => '&',
            "quot" => '"',
            "apos" => '\'',
            _ => {
                let number = match entity
                    .strip_prefix("#x")
                    .or_else(|| entity.strip_prefix("#X"))
                {
                    Some(hex) => u32::from_str_radix(hex, 16).ok(),
                    None => entity
                        .strip_prefix('#')
                        .and_then(|decimal| decimal.parse().ok()),
                };
                number
                    .and_then(char::from_u32)
                    .ok_or_else(|| broken(format!("&{entity}; is no character")))?
            }
        };
        said.push(character);
        rest = &after[end + 1..];
    }
    said.push_str(rest);
    Ok(said)
}
