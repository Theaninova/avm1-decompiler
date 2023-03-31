use crate::decompiler::read::read_str::read_str;
use swf::avm1::read::Reader;
use swf::avm1::types::{Push, Value};
use swf::error::{Error, Result};
use swf::extensions::ReadSwfExt;

/// Modified from [Reader::read_push].
/// LMD ActionScript, for some reason, decided to add all
/// strings to the constant pool aka symbols section
/// so the stock reader will crash on some
pub fn read_push<'a>(
    reader: &mut Reader<'a>,
    length: usize,
    symbols: &'a [String],
) -> Result<Push<'a>> {
    // TODO: Verify correct version for complex types.
    let end_pos = (reader.get_ref().as_ptr() as usize + length) as *const u8;
    let mut values = Vec::with_capacity(4);
    while reader.get_ref().as_ptr() < end_pos {
        values.push(read_push_value(reader, symbols)?);
    }
    Ok(Push { values })
}

/// Modified from [Reader::read_push_value].
/// LMD ActionScript, for some reason, decided to add all
/// strings to the constant pool aka symbols section
/// so the stock reader will crash on some
fn read_push_value<'a>(reader: &mut Reader<'a>, symbols: &'a [String]) -> Result<Value<'a>> {
    let value = match reader.read_u8()? {
        0 => Value::Str(read_str(reader, symbols)?),
        1 => Value::Float(reader.read_f32()?),
        2 => Value::Null,
        3 => Value::Undefined,
        4 => Value::Register(reader.read_u8()?),
        5 => Value::Bool(reader.read_u8()? != 0),
        6 => Value::Double(read_f64_me(reader)?),
        7 => Value::Int(reader.read_i32()?),
        8 => Value::ConstantPool(reader.read_u8()?.into()),
        9 => Value::ConstantPool(reader.read_u16()?),
        _ => return Err(Error::invalid_data("Invalid value type in ActionPush")),
    };
    Ok(value)
}

/// Taken from [Reader::read_f64_me] because it's private
#[inline]
fn read_f64_me(reader: &mut Reader) -> Result<f64> {
    // Flash weirdly stores (some?) f64 as two LE 32-bit chunks.
    // First word is the hi-word, second word is the lo-word.
    Ok(f64::from_bits(reader.read_u64()?.rotate_left(32)))
}
