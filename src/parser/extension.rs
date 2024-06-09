use std::fs::File;
use std::io::Write;

use super::struct_namer;


pub fn extension_writer(out_file: &mut File, key: &str) {
    let ext_struct_name = struct_namer(key);

    writeln!(out_file, "\n\n#[derive(Debug, Clone, Deserialize, Serialize)]").unwrap();
    writeln!(out_file, "#[serde(rename_all = \"PascalCase\")]").unwrap();
    writeln!(out_file, "pub struct {} {{}}", ext_struct_name).unwrap();
}