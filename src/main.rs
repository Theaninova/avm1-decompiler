mod test_constants;
mod ast;
mod decompiler;

use std::fs;
use swf::avm1::read::Reader;
use swf::extensions::ReadSwfExt;
use crate::decompiler::Avm1Decompiler;
use crate::test_constants::LMD_SYMBOLS;

fn main() {
    let data = fs::read(PATH).unwrap();
    let mut reader = Reader::new(&data, 1);

    let num_actions = reader.read_u32();

    let action_size = reader.read_u16().unwrap();
    reader.read_u16().unwrap();
    let action = reader.read_slice(action_size as usize - 2).unwrap();

    let mut test = Avm1Decompiler::new(action, LMD_SYMBOLS);
    let result = test.decompile().unwrap();

    for stmt in result {
        print!("{}", stmt);
    }
}

const PATH: &str = "E:\\Games\\W5X\\data_jp\\flash\\DRIVE\\Race\\Meter\\METER_NORMAL_D000\\METER_NORMAL_D000.abc";
