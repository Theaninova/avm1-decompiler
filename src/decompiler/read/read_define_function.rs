use crate::decompiler::read::read_str::read_str;
use std::num::NonZeroU8;
use swf::avm1::read::Reader;
use swf::avm1::types::{DefineFunction, DefineFunction2, FunctionFlags, FunctionParam};
use swf::error::Error;
use swf::extensions::ReadSwfExt;

pub fn read_define_function<'a>(
    reader: &mut Reader<'a>,
    action_length: &mut usize,
    symbols: &'a [String],
) -> Result<DefineFunction<'a>, Error> {
    let name = read_str(reader, symbols)?;
    let num_params = reader.read_u16()?;
    let mut params = Vec::with_capacity(num_params as usize);
    for _ in 0..num_params {
        params.push(read_str(reader, symbols)?);
    }
    // code_length isn't included in the DefineFunction's action length.
    let code_length: usize = (reader.read_u16()?).into();
    *action_length += code_length;
    Ok(DefineFunction {
        name,
        params,
        actions: reader.read_slice(code_length)?,
    })
}

pub fn read_define_function_2<'a>(
    reader: &mut Reader<'a>,
    action_length: &mut usize,
    symbols: &'a [String],
) -> Result<DefineFunction2<'a>, Error> {
    let name = read_str(reader, symbols)?;
    let num_params = reader.read_u16()?;
    let register_count = reader.read_u8()?;
    let flags = FunctionFlags::from_bits_truncate(reader.read_u16()?);
    let mut params = Vec::with_capacity(num_params as usize);
    for _ in 0..num_params {
        params.push(FunctionParam {
            register_index: NonZeroU8::new(reader.read_u8()?),
            name: read_str(reader, symbols)?,
        });
    }
    // code_length isn't included in the DefineFunction's length.
    let code_length: usize = (reader.read_u16()?).into();
    *action_length += code_length;
    Ok(DefineFunction2 {
        name,
        params,
        register_count,
        flags,
        actions: reader.read_slice(code_length)?,
    })
}
