mod array;
mod component;
mod reference;
mod extension;

use std::fs::{File, create_dir_all};
use std::path::PathBuf;
use std::io::Write;


pub fn component_structifier<P>(in_file: PathBuf, out_path: P) -> (String, Vec<String>, Vec<(PathBuf, (PathBuf, String))>)
where PathBuf: From<P> {
    let contents = std::fs::read_to_string(&in_file)
        .expect("Failed to read file");

    let value: serde_json::Value = serde_json::from_str(&contents)
        .expect("Failed to parse JSON");

    let definitions = value.get("definitions")
        .expect("No definitions");

    let out_subdir = in_file.file_name()
        .expect("No file name")
        .to_str()
        .expect("File name is not a string")
        .split_once(".json")
        .expect("File name is not in the correct format")
        .0;

    let out_root = PathBuf::from(out_path);
    let mut out_dir = out_root.clone();
    let schema_name = filenamer(out_subdir);
    out_dir.push(&schema_name);

    create_dir_all(&out_dir).expect("Failed to create out directory");

    // Run parsers and hope they generate the correct files
    let formats = component::component_parser(definitions, &out_dir, &schema_name);
    let post_processables = reference::reference_parser(definitions, &out_root, &out_dir);
    
    // Create mod.rs file in core_components directory
    let mut mod_file = out_dir.clone();
    mod_file.push("mod.rs");
    
    let mut mod_file = std::fs::File::create(&mod_file)
    .expect("Failed to create file");

    // Check if components dir is present
    let mut components_dir = out_dir.clone();
    components_dir.push("components");

    // Same for the extensions file
    let mut exts_file = out_dir.clone();
    exts_file.push("extensions.rs");

    // Same for the references file
    let mut refs_file = out_dir.clone();
    refs_file.push("references.rs");

    if components_dir.exists() {
        writeln!(mod_file, "mod components;").unwrap();
    }

    if exts_file.exists() {
        writeln!(mod_file, "mod extensions;").unwrap();
    }

    if refs_file.exists() {
        writeln!(mod_file, "mod references;").unwrap();
    }

    if exts_file.exists() || refs_file.exists() || components_dir.exists() {
        writeln!(mod_file, "\n").unwrap();
    }

    if exts_file.exists() {
        writeln!(mod_file, "pub use extensions::*;").unwrap();
    }

    if refs_file.exists() {
        writeln!(mod_file, "pub use references::*;").unwrap();
    }

    if components_dir.exists() {
        writeln!(mod_file, "pub use components::*;").unwrap();
    }

    (schema_name, formats, post_processables)
}

pub fn struct_namer<T>(key: T) -> String where T: AsRef<str> {
    key.as_ref().replace("_", "")
}

pub fn enum_namer<T>(key: T) -> String where T: AsRef<str> {
    let key = key.as_ref();
    let mut uc_next = false;
    let mut enum_name = vec![];

    for (i, s) in key.chars().enumerate() {
        // Non alphanumeric char, treat it as a word break
        if !s.is_ascii_alphanumeric() {
            uc_next = true;
            continue;
        }

        // Uppercase initial char
        if i == 0 {
            enum_name.extend(s.to_uppercase());
            continue;
        }

        // Previous char was in uppercase so don't add _
        if uc_next {
            enum_name.extend(s.to_uppercase());
            uc_next = false;
            continue;
        }

        enum_name.push(s);
    }

    enum_name.into_iter().collect::<String>()
}

pub fn snaker<T>(key: T) -> String where T: AsRef<str> {
    // Rename _ key to _uc
    let key = key.as_ref();
    
    if key.eq("_") {
        return "_uc".to_string()
    }

    let mut mod_name = vec![];
    let mut prev_uc = false;
    let mut prev_underscore = false;

    for (i, s) in key.chars().enumerate() {
        // Underscore char, take it, flip boolean and proceed
        if s.eq(&'_') {
            // Don't add multiple underscores in a row
            if prev_underscore {
                continue;
            }

            mod_name.push(s);
            prev_underscore = true;
            continue;
        }

        // Lowecase char, take it and proceed
        if s.is_lowercase() {
            mod_name.push(s);
            prev_uc = false;
            continue;
        }

        // Previous char was in uppercase so don't add _
        if prev_uc {
            mod_name.extend(s.to_lowercase());
            continue;
        }

        // Uppercase initial char, don't add underscore yet
        if i == 0 {
            mod_name.extend(s.to_lowercase());
            prev_uc = true;
            continue;
        }

        // Now we need that undescore
        mod_name.push('_');
        mod_name.extend(s.to_lowercase());
        prev_uc = true;
    }

    mod_name.into_iter().collect::<String>()
}

pub fn filenamer<T>(name: T) -> String where T: AsRef<str> {
    let mut file = name.as_ref().to_lowercase();

    if file.ends_with(".json") {
        file = file.replace(".json", "");
    }

    // Replace - with _
    file = file.replace("-", "_");

    // Replace . with _
    file = file.replace(".", "_");

    file.replace("__", "_")
}

pub fn impl_title_and_descr_writer(title: Option<&str>, description: Option<&str>, out_file: &mut File) {
    // make impl for the struct with methods returning static title and description
    if let Some(s) = title {
        // writeln!(out_file, "    #[allow(dead_code)]").unwrap();
        writeln!(out_file, "    pub fn title() -> &'static str {{").unwrap();
        writeln!(out_file, "        \"{}\"", s).unwrap();
        writeln!(out_file, "    }}").unwrap();
    }

    if let Some(s) = description {
        // writeln!(out_file, "    #[allow(dead_code)]").unwrap();
        writeln!(out_file, "    pub fn description() -> &'static str {{").unwrap();
        writeln!(out_file, "        \"{}\"", s).unwrap();
        writeln!(out_file, "    }}").unwrap();
    }
}
