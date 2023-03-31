use crate::decompiler::read::read_define_function::{read_define_function, read_define_function_2};
use crate::decompiler::read::read_push::read_push;
use swf::avm1::read::Reader;
use swf::avm1::types::Action;
use swf::error::Error;
use swf::extensions::ReadSwfExt;

mod read_define_function;
mod read_push;
mod read_str;

pub fn read<'a>(
    reader: &mut Reader<'a>,
    input: &'a [u8],
    symbols: &'a [String],
) -> Result<Action<'a>, Error> {
    let pos = reader.pos(input);
    let opcode = reader.read_u8()?;
    let mut length: usize = if opcode >= 0x80 {
        reader.read_u16()?.into()
    } else {
        0
    };

    match opcode {
        0x96 => Ok(Action::Push(read_push(reader, length, symbols)?)),
        0x8e => Ok(Action::DefineFunction2(read_define_function_2(
            reader,
            &mut length,
            symbols,
        )?)),
        0x9b => Ok(Action::DefineFunction(read_define_function(
            reader,
            &mut length,
            symbols,
        )?)),
        _ => {
            reader.seek_absolute(input, pos);
            reader.read_action()
        }
    }
}
