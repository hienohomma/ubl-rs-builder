use serde_json::Value;
use std::fs::remove_file;
use std::vec;
use std::{fs::File, io::Write};
use std::path::PathBuf;

use crate::parser::impl_title_and_descr_writer;

use super::{filenamer, snaker, struct_namer};


// Basic reference is an object with "$ref" key and optionally title and description.
// Example simple:
// "ModelName": {
//     "$ref": "BDNDR-UnqualifiedDataTypes-1.1.json#/definitions/NameType"
//     },
// Example with title and description:
// "CodeType": {
//     "title": "Code. Type",
//     "description": "A character string (letters, figures, or symbols) that for brevity and/or language independence may be used to represent or replace a definitive value or text of an attribute, together with relevant supplementary information.",
//     "$ref": "BDNDR-CCTS_CCT_SchemaModule-1.1.json#/definitions/CodeType"
//   },

pub fn reference_parser(definitions: &Value, out_root: &PathBuf, out_dir: &PathBuf) -> Vec<(PathBuf, (PathBuf, String))> {
    let mut refs_file = out_dir.clone();
    refs_file.push("references.rs");

    // Delete old before writing new
    remove_file(&refs_file).ok();

    // Collect references we have to process once all schemas are processed
    let mut post_process_refs = vec![];

    for (k, v) in definitions.as_object().expect("Definitions is not an object") {
        // Check if object has "$ref" key
        let obj = v.as_object().expect("Object is not an object");

        if !obj.contains_key("$ref") {
            continue;
        }

        //  Optional values
        let title = obj.get("title").and_then(|v|v.as_str());
        let description = obj.get("description").and_then(|v|v.as_str());

        // Required values
        let ref_path = obj.get("$ref")
            .expect(&format!("Reference does not have $ref key: {:?}", v))
            .as_str()
            .expect("Reference is not a string");

        let (subdir, mod_name, struct_name) = ref_path_splitter(ref_path);

        // Open file for writing, but dont overwrite it
        let mut out_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&refs_file)
            .expect("Failed to open mod reference file");

        let ref_struct_name = struct_namer(k);

        match title.is_none() && description.is_none() {
            true => {
                // Write type alias for referenced struct
                if let Some(t) = type_alias_writer(&mut out_file, out_root, out_dir, subdir, &mod_name, struct_name, ref_struct_name) {
                    post_process_refs.push((refs_file.to_owned(), t));
                }
            },
            false => {
                // Write struct with title and description
                reference_struct_writer(&mut out_file, &subdir, &struct_name, &ref_struct_name);
                    
                writeln!(out_file, "\nimpl {} {{", ref_struct_name).unwrap();
                impl_title_and_descr_writer(title, description, &mut out_file);
                writeln!(out_file, "}}").unwrap();
            }
        }
    }

    post_process_refs
}

pub fn ref_path_splitter<'a>(ref_path: &'a str) -> (Option<String>, String, String) {
    let (mod_dir, struct_name) = match ref_path.split_once(".json#/definitions/") {
        Some(t) => (Some(t.0), t.1),
        None => (
            None,
            ref_path.strip_prefix("#/definitions/").expect("Local reference is not in the correct format"),
        )
    };

    let subdir = mod_dir.and_then(|s|Some(filenamer(s)));
    let mod_snake_name = snaker(struct_name);
    let struct_name = struct_namer(struct_name);

    (subdir, mod_snake_name, struct_name)
}

fn type_alias_writer(out_file: &mut File, out_root: &PathBuf, out_dir: &PathBuf, subdir: Option<String>,
mod_name: &String, struct_name: String, ref_struct_name: String) -> Option<(PathBuf, String)> {
    // Write type alias for referenced struct
    match subdir {
        Some(s) => {
            let mut referenced_path = out_root.clone();

            referenced_path.push(&s);
            referenced_path.push("components");
            referenced_path.push(mod_name);
            referenced_path.set_extension("rs");

            let comp_ref = format!(
                "pub type {} = crate::{}::{};", ref_struct_name, s, struct_name
            );

            return Some((referenced_path.to_owned(), /*alias_ref, */comp_ref))
        },
        None => {
            let mut referenced_path = out_dir.clone();
            referenced_path.push("components");
            referenced_path.push(mod_name);
            referenced_path.set_extension("rs");

            writeln!(out_file, "pub type {} = super::{};",
                ref_struct_name, struct_name
            )
        },
    }.unwrap();

    None
}


fn reference_struct_writer(out_file: &mut File, subdir: &Option<String>, struct_name: &String, ref_struct_name: &String) {
    let ref_target = match subdir {
        Some(s) => format!("crate::{}::{}", s, struct_name),
        None => format!("super::{}", struct_name),
    };

    // Write struct with title and description
    writeln!(out_file, "#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]").unwrap();
    writeln!(out_file, "#[serde(transparent)]").unwrap();
    writeln!(out_file, "pub struct {} (pub {});", ref_struct_name, ref_target).unwrap();
}
