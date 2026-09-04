//! Streaming protobuf decoder using borrowed payload slices. Unknown fields
//! are skipped without allocating; nested messages borrow the original buffer.
//! Malformed tails and wire-type mismatches are ignored, as in the reference.

use serde_json::{Map, Value};
use thiserror::Error;

use super::schema::{ProtoType, Schema};

#[derive(Debug, Error)]
#[error("{0}")]
pub struct ProtoError(pub String);

enum RawData<'a> {
    Varint(u64),
    Bytes(&'a [u8]),
}

/// Protobuf int64 values can occupy ten bytes, including negative values.
fn read_varint(buf: &[u8], offset: &mut usize) -> Option<u64> {
    let mut value = 0;
    for shift in (0..70).step_by(7) {
        let byte = *buf.get(*offset)?;
        *offset += 1;
        if shift == 63 && byte > 1 {
            return None;
        }
        value |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Some(value);
        }
    }
    None
}

fn read_field<'a>(buf: &'a [u8], offset: &mut usize) -> Option<(u32, u8, RawData<'a>)> {
    let key = read_varint(buf, offset)?;
    let tag = key >> 3;
    if tag == 0 || tag >= 1 << 29 {
        return None;
    }
    let wire = (key & 7) as u8;
    let data = if wire == 0 {
        RawData::Varint(read_varint(buf, offset)?)
    } else {
        let len = match wire {
            1 => 8,
            2 => usize::try_from(read_varint(buf, offset)?).ok()?,
            5 => 4,
            _ => return None,
        };
        let end = offset.checked_add(len)?;
        let bytes = buf.get(*offset..end)?;
        *offset = end;
        RawData::Bytes(bytes)
    };
    Some((tag as u32, wire, data))
}

fn parse(data: RawData<'_>, wire: u8, ty: ProtoType) -> Option<Value> {
    match (ty, wire, data) {
        (ProtoType::Int | ProtoType::Long, 0, RawData::Varint(v)) => Some(Value::from(v as i64)),
        (ProtoType::Bool, 0, RawData::Varint(v)) => Some(Value::Bool(v == 1)),
        (ProtoType::String, 2, RawData::Bytes(b)) => Some(Value::String(String::from_utf8_lossy(b).into_owned())),
        (ProtoType::Message(sub), 2, RawData::Bytes(b)) => decode(b, sub).ok(),
        (ProtoType::Float, 5, RawData::Bytes(b)) => Some(Value::from(f32::from_le_bytes(b.try_into().ok()?))),
        _ => None,
    }
}

/// Decodes known fields in wire order. Repeated values preserve source order;
/// singular fields retain the last valid occurrence.
pub fn decode(buf: &[u8], schema: &Schema) -> Result<Value, ProtoError> {
    let mut result = Map::new();
    let mut offset = 0;
    while let Some((tag, wire, data)) = read_field(buf, &mut offset) {
        let Some((_, field)) = schema.fields.iter().find(|(number, _)| *number == tag) else {
            continue;
        };
        let value = parse(data, wire, field.ty);
        if field.repeated {
            // Preserve the reference's empty array for a present field whose
            // occurrences all have the wrong wire type.
            let values = result.entry(field.name).or_insert_with(|| Value::Array(Vec::new()));
            if let (Value::Array(values), Some(value)) = (values, value) {
                values.push(value);
            }
        } else if let Some(value) = value {
            result.insert(field.name.to_string(), value);
        }
    }
    Ok(Value::Object(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::schema::field;
    use serde_json::json;

    static CHILD: Schema = Schema {
        fields: &[(1, field("id", ProtoType::Long, false))],
    };
    static SCHEMA: Schema = Schema {
        fields: &[
            (1, field("id", ProtoType::Long, false)),
            (2, field("names", ProtoType::String, true)),
            (3, field("child", ProtoType::Message(&CHILD), false)),
            (4, field("value", ProtoType::Float, false)),
        ],
    };

    #[test]
    fn preserves_order_last_valid_value_and_nested_messages() {
        let buf = [8, 1, 18, 1, b'a', 8, 2, 18, 1, b'b', 10, 1, 0, 26, 2, 8, 3];
        assert_eq!(
            decode(&buf, &SCHEMA).unwrap(),
            json!({"id": 2, "names": ["a", "b"], "child": {"id": 3}})
        );
        assert_eq!(decode(&[16, 1], &SCHEMA).unwrap(), json!({"names": []}));
    }

    #[test]
    fn accepts_ten_byte_signed_varints_and_rejects_overflow() {
        let mut negative = vec![8];
        negative.extend_from_slice(&[255; 9]);
        negative.push(1);
        assert_eq!(decode(&negative, &SCHEMA).unwrap(), json!({"id": -1}));
        *negative.last_mut().unwrap() = 2;
        assert_eq!(decode(&negative, &SCHEMA).unwrap(), json!({}));
        assert_eq!(decode(&[8, 128], &SCHEMA).unwrap(), json!({}));
    }

    #[test]
    fn skips_unknown_fields_and_truncated_or_overflowing_lengths() {
        let mut buf = vec![42, 3, 1, 2, 3, 8, 7, 18];
        buf.extend_from_slice(&[255; 9]);
        buf.push(1);
        assert_eq!(decode(&buf, &SCHEMA).unwrap(), json!({"id": 7}));
        assert_eq!(decode(&[8, 7, 18, 3, b'a'], &SCHEMA).unwrap(), json!({"id": 7}));
    }

    #[test]
    fn decodes_float_and_skips_fixed64() {
        let mut buf = vec![41];
        buf.extend_from_slice(&[0; 8]);
        buf.push(37);
        buf.extend_from_slice(&1.25f32.to_le_bytes());
        assert_eq!(decode(&buf, &SCHEMA).unwrap(), json!({"value": 1.25}));
    }
}
