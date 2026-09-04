//! A generic protobuf wire-format decoder ported from GarupaSpeedTracker's
//! `GarupaParser`. It reads tag/wire-type/length-value fields from raw binary
//! and maps them to JSON values according to a runtime [`Schema`] descriptor,
//! so no precompiled `.proto` files are needed.
//!
//! Behavior mirrors the reference implementation:
//! - Wire types are validated against the schema; mismatches are skipped.
//! - Repeated fields produce arrays; non-repeated fields take the last valid
//!   occurrence, which is robust to trailing garbage.

use std::collections::HashMap;

use serde_json::{Map, Value};
use thiserror::Error;

use super::schema::{ProtoType, Schema};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WireType {
    Varint,
    Fixed64,
    LengthDelimited,
    Fixed32,
}

#[derive(Debug, Clone)]
enum RawData {
    Varint(u64),
    Bytes(Vec<u8>),
}

#[derive(Debug, Clone)]
struct RawField {
    field: u32,
    wire_type: WireType,
    data: RawData,
}

#[derive(Debug, Error)]
#[error("{0}")]
pub struct ProtoError(pub String);

/// Reads a base-128 varint from `buf` starting at `offset`.
/// Returns the decoded value and the new offset, or `None` on truncation or overflow.
fn read_varint(buf: &[u8], offset: usize) -> Option<(u64, usize)> {
    let mut value: u64 = 0;
    let mut shift: u32 = 0;
    let mut cursor = offset;

    while cursor < buf.len() {
        let byte = buf[cursor];
        cursor += 1;
        value |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Some((value, cursor));
        }
        shift += 7;
        if shift > 56 {
            return None;
        }
    }

    None
}

/// Parses the whole buffer into a flat list of raw protobuf fields.
/// Malformed tail bytes are silently ignored, matching the reference.
fn parse_raw_fields(buf: &[u8]) -> Vec<RawField> {
    let mut results = Vec::new();
    let mut offset = 0;

    while offset < buf.len() {
        let (key, next) = match read_varint(buf, offset) {
            Some(v) => v,
            None => break,
        };
        offset = next;
        if key == 0 {
            break;
        }

        let field = (key >> 3) as u32;
        let wire_type = key & 0x07;
        if field == 0 {
            break;
        }

        match wire_type {
            0 => {
                let (value, new_offset) = match read_varint(buf, offset) {
                    Some(v) => v,
                    None => break,
                };
                offset = new_offset;
                results.push(RawField {
                    field,
                    wire_type: WireType::Varint,
                    data: RawData::Varint(value),
                });
            }
            2 => {
                let (len, new_offset) = match read_varint(buf, offset) {
                    Some(v) => v,
                    None => break,
                };
                offset = new_offset;
                let len = len as usize;
                if offset + len > buf.len() {
                    break;
                }
                let inner = buf[offset..offset + len].to_vec();
                offset += len;
                results.push(RawField {
                    field,
                    wire_type: WireType::LengthDelimited,
                    data: RawData::Bytes(inner),
                });
            }
            1 => {
                if offset + 8 > buf.len() {
                    break;
                }
                let inner = buf[offset..offset + 8].to_vec();
                offset += 8;
                results.push(RawField {
                    field,
                    wire_type: WireType::Fixed64,
                    data: RawData::Bytes(inner),
                });
            }
            5 => {
                if offset + 4 > buf.len() {
                    break;
                }
                let inner = buf[offset..offset + 4].to_vec();
                offset += 4;
                results.push(RawField {
                    field,
                    wire_type: WireType::Fixed32,
                    data: RawData::Bytes(inner),
                });
            }
            _ => break,
        }
    }

    results
}

/// Decodes a protobuf buffer into a JSON object according to `schema`.
pub fn decode(buf: &[u8], schema: &Schema) -> Result<Value, ProtoError> {
    let raw_fields = parse_raw_fields(buf);

    let mut groups: HashMap<u32, Vec<RawField>> = HashMap::new();
    for field in raw_fields {
        groups.entry(field.field).or_default().push(field);
    }

    let mut result = Map::new();

    for (tag, field_def) in schema.fields {
        let items = match groups.get(tag) {
            Some(items) if !items.is_empty() => items,
            _ => continue,
        };

        let parse = |item: &RawField| -> Option<Value> {
            let wt = item.wire_type;
            match field_def.ty {
                ProtoType::Int | ProtoType::Long => {
                    if wt != WireType::Varint {
                        return None;
                    }
                    match &item.data {
                        RawData::Varint(v) => Some(Value::from(*v as i64)),
                        _ => None,
                    }
                }
                ProtoType::Bool => {
                    if wt != WireType::Varint {
                        return None;
                    }
                    match &item.data {
                        RawData::Varint(v) => Some(Value::Bool(*v == 1)),
                        _ => None,
                    }
                }
                ProtoType::String => {
                    if wt != WireType::LengthDelimited {
                        return None;
                    }
                    match &item.data {
                        RawData::Bytes(b) => Some(Value::String(String::from_utf8_lossy(b).into_owned())),
                        _ => None,
                    }
                }
                ProtoType::Message(sub) => {
                    if wt != WireType::LengthDelimited {
                        return None;
                    }
                    match &item.data {
                        RawData::Bytes(b) => decode(b, sub).ok(),
                        _ => None,
                    }
                }
                ProtoType::Float => {
                    if wt != WireType::Fixed32 {
                        return None;
                    }
                    match &item.data {
                        RawData::Bytes(b) if b.len() == 4 => {
                            let arr: [u8; 4] = b.as_slice().try_into().ok()?;
                            Some(Value::from(f32::from_le_bytes(arr)))
                        }
                        _ => None,
                    }
                }
            }
        };

        if field_def.repeated {
            let arr: Vec<Value> = items.iter().filter_map(&parse).collect();
            result.insert(field_def.name.to_string(), Value::Array(arr));
        } else {
            for item in items.iter().rev() {
                if let Some(v) = parse(item) {
                    result.insert(field_def.name.to_string(), v);
                    break;
                }
            }
        }
    }

    Ok(Value::Object(result))
}
