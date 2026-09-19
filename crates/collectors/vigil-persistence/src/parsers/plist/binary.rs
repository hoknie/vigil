use std::cell::Cell;
use std::collections::BTreeMap;

use super::property_list::DEEPEST;
use super::refusal::PlistRefusal;
use super::value::PlistValue;

const HEADER: usize = 8;

const TRAILER: usize = 32;

const SECONDS_FROM_1970_TO_2001: f64 = 978_307_200.0;

pub(super) const MOST_OBJECTS_VISITED: usize = 200_000;

struct Document<'a> {
    bytes: &'a [u8],
    offsets: Vec<usize>,
    reference_size: usize,
    visited: Cell<usize>,
}

pub(super) fn parse(bytes: &[u8]) -> Result<PlistValue, PlistRefusal> {
    if bytes.len() < HEADER + TRAILER {
        return Err(broken("shorter than a header and a trailer"));
    }
    let trailer = &bytes[bytes.len() - TRAILER..];
    let offset_size = usize::from(trailer[6]);
    let reference_size = usize::from(trailer[7]);
    let objects = number(&trailer[8..16]);
    let top = number(&trailer[16..24]);
    let table = number(&trailer[24..32]);

    if !(1..=8).contains(&offset_size) || !(1..=8).contains(&reference_size) {
        return Err(broken("the sizes in the trailer are not sizes"));
    }
    let objects = usize::try_from(objects).map_err(|_| broken("too many objects"))?;
    let table = usize::try_from(table).map_err(|_| broken("the offset table is past the end"))?;
    if objects > bytes.len()
        || table < HEADER
        || table
            .checked_add(objects.saturating_mul(offset_size))
            .is_none_or(|end| end > bytes.len() - TRAILER)
    {
        return Err(broken("the offset table does not fit in the file"));
    }

    let offsets = (0..objects)
        .map(|index| {
            let at = table + index * offset_size;
            usize::try_from(number(&bytes[at..at + offset_size])).unwrap_or(usize::MAX)
        })
        .collect();
    let document = Document {
        bytes,
        offsets,
        reference_size,
        visited: Cell::new(0),
    };
    let top = usize::try_from(top).map_err(|_| broken("the top object is not in the file"))?;
    document.object(top, 0)
}

fn broken(why: &str) -> PlistRefusal {
    PlistRefusal::Broken(format!("binary property list: {why}"))
}

fn number(bytes: &[u8]) -> u64 {
    bytes
        .iter()
        .fold(0u64, |held, byte| (held << 8) | u64::from(*byte))
}

impl Document<'_> {
    fn at(&self, from: usize, length: usize) -> Result<&[u8], PlistRefusal> {
        let end = from
            .checked_add(length)
            .filter(|end| *end <= self.bytes.len() - TRAILER)
            .ok_or_else(|| broken("an object runs past the end"))?;
        Ok(&self.bytes[from..end])
    }

    fn object(&self, index: usize, depth: usize) -> Result<PlistValue, PlistRefusal> {
        if depth > DEEPEST {
            return Err(PlistRefusal::TooDeep);
        }
        self.visited.set(self.visited.get() + 1);
        if self.visited.get() > MOST_OBJECTS_VISITED {
            return Err(broken(
                "more objects are reached than the file holds room for; one is referred to over and over",
            ));
        }
        let offset = *self
            .offsets
            .get(index)
            .ok_or_else(|| broken("a reference to an object that is not there"))?;
        let marker = self.at(offset, 1)?[0];
        let (kind, low) = (marker >> 4, marker & 0x0f);

        match kind {
            0x0 => match low {
                0x8 => Ok(PlistValue::Boolean(false)),
                0x9 => Ok(PlistValue::Boolean(true)),
                _ => Err(broken("a null or fill where a value was due")),
            },
            0x1 => {
                let size = 1usize << low;
                if size > 16 {
                    return Err(broken("an integer wider than sixteen bytes"));
                }
                let raw = self.at(offset + 1, size)?;
                let value = raw
                    .iter()
                    .fold(0u128, |held, byte| (held << 8) | u128::from(*byte));
                Ok(PlistValue::Integer(match size {
                    8 => i128::from(value as u64 as i64),
                    16 => value as i128,
                    _ => value as i128,
                }))
            }
            0x2 => {
                let size = 1usize << low;
                let raw = self.at(offset + 1, size)?;
                match size {
                    4 => Ok(PlistValue::Real(f64::from(f32::from_be_bytes(
                        raw.try_into().map_err(|_| broken("a short real"))?,
                    )))),
                    8 => Ok(PlistValue::Real(f64::from_be_bytes(
                        raw.try_into().map_err(|_| broken("a short real"))?,
                    ))),
                    _ => Err(broken("a real of an odd width")),
                }
            }
            0x3 => {
                let raw = self.at(offset + 1, 8)?;
                let seconds =
                    f64::from_be_bytes(raw.try_into().map_err(|_| broken("a short date"))?);
                Ok(PlistValue::Date(format!(
                    "{}",
                    (seconds + SECONDS_FROM_1970_TO_2001) as i64
                )))
            }
            0x4 => {
                let (length, start) = self.length(offset, low)?;
                self.at(start, length)?;
                Ok(PlistValue::Data(length))
            }
            0x5 => {
                let (length, start) = self.length(offset, low)?;
                let raw = self.at(start, length)?;
                Ok(PlistValue::String(
                    String::from_utf8_lossy(raw).into_owned(),
                ))
            }
            0x6 => {
                let (length, start) = self.length(offset, low)?;
                let raw = self.at(
                    start,
                    length.checked_mul(2).ok_or_else(|| broken("too long"))?,
                )?;
                let units: Vec<u16> = raw
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|pair| u16::from_be_bytes(*pair))
                    .collect();
                Ok(PlistValue::String(String::from_utf16_lossy(&units)))
            }
            0x8 => {
                let raw = self.at(offset + 1, usize::from(low) + 1)?;
                Ok(PlistValue::Uid(number(raw)))
            }
            0xA | 0xC => {
                let (length, start) = self.length(offset, low)?;
                let references = self.references(start, length)?;
                let mut values = Vec::with_capacity(length.min(4096));
                for reference in references {
                    values.push(self.object(reference, depth + 1)?);
                }
                Ok(PlistValue::Array(values))
            }
            0xD => {
                let (length, start) = self.length(offset, low)?;
                let keys = self.references(start, length)?;
                let values = self.references(start + length * self.reference_size, length)?;
                let mut entries = BTreeMap::new();
                for (key, value) in keys.into_iter().zip(values) {
                    let key = match self.object(key, depth + 1)? {
                        PlistValue::String(key) => key,
                        _ => return Err(broken("a dictionary key that is not a string")),
                    };
                    entries.insert(key, self.object(value, depth + 1)?);
                }
                Ok(PlistValue::Dictionary(entries))
            }
            _ => Err(broken("an object of a kind no property list has")),
        }
    }

    fn length(&self, offset: usize, low: u8) -> Result<(usize, usize), PlistRefusal> {
        if low != 0x0f {
            return Ok((usize::from(low), offset + 1));
        }
        let marker = self.at(offset + 1, 1)?[0];
        if marker >> 4 != 0x1 {
            return Err(broken("a length that is not an integer"));
        }
        let size = 1usize << (marker & 0x0f);
        if size > 8 {
            return Err(broken("a length wider than eight bytes"));
        }
        let raw = self.at(offset + 2, size)?;
        let length = usize::try_from(number(raw)).map_err(|_| broken("a length too large"))?;
        if length > self.bytes.len() {
            return Err(broken("a length longer than the file"));
        }
        Ok((length, offset + 2 + size))
    }

    fn references(&self, start: usize, count: usize) -> Result<Vec<usize>, PlistRefusal> {
        let raw = self.at(
            start,
            count
                .checked_mul(self.reference_size)
                .ok_or_else(|| broken("too many references"))?,
        )?;
        Ok(raw
            .chunks_exact(self.reference_size)
            .map(|chunk| usize::try_from(number(chunk)).unwrap_or(usize::MAX))
            .collect())
    }
}
