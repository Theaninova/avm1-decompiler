pub mod ast;
pub mod decompiler;
pub mod emitter;
pub mod test_constants;

use crate::decompiler::{decompile, VmData};
use crate::test_constants::LMD_SYMBOLS;
use std::fs;
use std::path::Path;
use swf::avm1::read::Reader;
use swf::extensions::ReadSwfExt;

fn main() {
    let path = Path::new("E:\\Games\\W5X\\data_jp\\flash\\DRIVE\\Race\\Meter\\METER_NORMAL_D000\\METER_NORMAL_D000.abc");
    let data = fs::read(path).unwrap();
    let mut reader = Reader::new(&data, 1);

    let num_actions = reader.read_u32().unwrap();

    let action_size = reader.read_u16().unwrap();
    reader.read_u16().unwrap();
    let action = reader.read_slice(action_size as usize - 2).unwrap();

    let pool: Vec<String> = LMD_SYMBOLS.iter().map(|it| it.to_string()).collect();
    let result = decompile(VmData {
        bytecode: action,
        constant_pool: &pool,
        strict: false,
        registers: Vec::new(),
    })
    .unwrap();

    let emitted_code: Vec<String> = result.iter().map(|it| it.to_string()).collect();
    fs::write(path.with_extension("as"), emitted_code.join("")).unwrap();
}
