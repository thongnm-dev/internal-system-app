use crate::app::result::AppResult;
use encoding_rs::SHIFT_JIS;
use std::fs;
use std::path::Path;

pub fn read_shift_jis_text(path: &Path) -> AppResult<String> {
    let bytes = fs::read(path)?;
    let (content, _, _) = SHIFT_JIS.decode(&bytes);
    Ok(content.into_owned())
}
