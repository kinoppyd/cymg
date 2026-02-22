use std::error::Error;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

pub(crate) fn build_output_path(input_path: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let stem = input_path
        .file_stem()
        .ok_or("input image must have a file name")?;
    let ext = input_path
        .extension()
        .ok_or("input image must have an extension")?;

    let mut output_name = OsString::new();
    output_name.push(stem);
    output_name.push(".cyber.");
    output_name.push(ext);

    let parent = input_path.parent().unwrap_or(Path::new("."));
    Ok(parent.join(output_name))
}
