use swf::avm1::read::Reader;
use swf::error::{Error, Result};
use swf::extensions::ReadSwfExt;
use swf::SwfStr;

#[inline]
pub fn read_str<'a>(reader: &mut Reader<'a>, symbols: &'a [String]) -> Result<&'a SwfStr> {
    let id = reader.read_u16()?;
    if let Some(result) = symbols.get(id as usize) {
        Ok(SwfStr::from_utf8_str(result))
    } else {
        Err(Error::InvalidData("Invalid string reference".into()))
    }
}
