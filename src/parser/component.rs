use serde_json::Value;
use std::collections::HashMap;
use std::fs::{create_dir_all, remove_dir_all, remove_file};
use std::io::Write;
use std::path::PathBuf;

use crate::parser::impl_title_and_descr_writer;

use super::array::array_struct_field_writer;
use super::extension::extension_writer;
use super::{enum_namer, snaker, struct_namer};

// Component object has a "type" key with value "object" and optional title, description and
// "additionalProperties" key with boolean value.
// It has to have "properties" key with object value and optional "required" key
// with array value, without those keys it is not an object.

// Properties are objects with "type" key and value of type of the property.
// Allowed values are: "string", "number", "integer", "boolean", "array", "object".

// Example of component object:
// "AmountType": {
//     "title": "Amount. Type",
//     "description": "A number of monetary units specified in a currency where the unit of the currency is explicit or implied.",
//     "required": [
//       "_"
//     ],
//     "properties": {
//       "_": {
//         "type": "number"
//       },
//       "currencyID": {
//         "type": "string"
//       },
//       "currencyCodeListVersionID": {
//         "type": "string"
//       }
//     },
//     "additionalProperties": false,
//     "type": "object"
//   },

// Example of a array as component:
// "ID": {
//     "title": "Activity Data Line. Identifier",
//     "description": "An identifier for this activity data line.",
//     "items": {
//       "$ref": "UBL-CommonBasicComponents-2.1.json#/definitions/ID"
//       },
//     "maxItems": 1,
//     "minItems": 1,
//     "type": "array"
//     },

const UC_ALPHABETS: [&str; 26] = [
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J",
    "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T",
    "U", "V", "W", "X", "Y", "Z"
];

pub fn component_parser(definitions: &Value, out_dir: &PathBuf, parent_mod: &str) -> Vec<String> {
    let mut comps_dir = out_dir.clone();
    comps_dir.push("components");

    let mut exts_file_path = out_dir.clone();
    exts_file_path.push("extensions.rs");

    // Delete old before writing new
    remove_dir_all(&comps_dir).ok();
    remove_file(&exts_file_path).ok();

    // Collect all mod names and string formats
    let mut mods = vec![];
    let mut formats = vec![];
    let mut extensions = false;

    create_dir_all(&comps_dir).expect("Failed to create components directory");
    let mut exts_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&exts_file_path)
        .expect(&format!("Failed to open extensions file: {:?}", exts_file_path));

    writeln!(exts_file, "use serde::{{Deserialize, Serialize}};").unwrap();

    for (k, v) in definitions.as_object().expect("Definitions is not an object") {
        let obj = v.as_object().expect("Object is not an object");
        let obj_type = obj.get("type").and_then(|v|v.as_str());

        if obj_type != Some("object") {
            continue;
        }

        if !obj.contains_key("properties") {
            extension_writer(&mut exts_file, k);
            extensions = true;
            continue;
        }

        // Required values from root level
        let properties = obj.get("properties")
            .unwrap()
            .as_object()
            .expect("Properties is not an object");

        //  Optional values from root level
        let title = obj.get("title").and_then(|v|v.as_str());
        let description = obj.get("description").and_then(|v|v.as_str());
        let additional_props = obj.get("additionalProperties")
            .and_then(|v|v.as_bool())
            .unwrap_or(false);

        let required = obj.get("required").and_then(|v|
            Some(v.as_array()
            .expect("Required is not an array")
            .into_iter()
            .map(|v|v.as_str().expect("Required array key is not a string"))
            .collect::<Vec<&str>>())
        ).unwrap_or(vec![]);

        let mut comp_file = comps_dir.clone();
        let filename = snaker(k);
        
        comp_file.push(&filename);
        comp_file.set_extension("rs");
        mods.push(filename);

        // Open file for writing, but dont overwrite it
        let mut out_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&comp_file)
            .expect(&format!("Failed to open component file: {:?}", comp_file));

        let comp_struct_name = struct_namer(k);
        let mut impl_buffer = vec![];
        let mut validations = HashMap::new();
        let mut new_instance = HashMap::new();

        writeln!(out_file, "use serde::{{Deserialize, Serialize}};").unwrap();
        writeln!(out_file, "use crate::{{UblError, Component, Componentable}};").unwrap();
        writeln!(out_file, "\n\n#[derive(Debug, Clone, Deserialize, Serialize)]").unwrap();

        writeln!(out_file, "pub struct {} {{", comp_struct_name).unwrap();

        // Components come from schema properties
        for (pk, pv) in properties {
            // Open component type to see what type of property it is
            let comp_type = pv.get("type")
                .expect("No type in property")
                .as_str()
                .expect("Type is not a string");

            // Some properties have format key in which case we should use 
            // FormattedValue enum for this component value
            let (string_format, format_variant) = match pv.get("format") {
                Some(v) => {
                    let format = v.as_str().expect("Format is not a string");
                    let mut enum_key = enum_namer(format);
                    
                    let enum_variant = format!("crate::FormattedValue::{}(s)", enum_key);
                    enum_key.push_str("(String)");
                    
                    if !formats.contains(&enum_key) {
                        formats.push(enum_key.to_owned());
                    }

                    ("crate::FormattedValue", Some(enum_variant))
                },
                None => ("String", None)
            };

            let snake_key = snaker(pk);
            let is_req = required.contains(&pk.as_str());

            // Serialize _uc as lowdash symbol in json
            if snake_key.eq("_uc") {
                writeln!(out_file, "    #[serde(rename = \"_\")]").unwrap();
            }
            // Other fields have to be serialized as they are in json spec
            else {
                writeln!(out_file, "    #[serde(rename = \"{}\")]", pk).unwrap();
            }

            match comp_type {
                "string" => {
                    match is_req {
                        true => {
                            writeln!(out_file, "    pub {}: {},", snake_key, string_format).unwrap()
                        },
                        false => {
                            writeln!(out_file, "    #[serde(skip_serializing_if = \"Option::is_none\")]").unwrap();
                            writeln!(out_file, "    pub {}: Option<{}>,", snake_key, string_format).unwrap()
                        }
                    }

                    let val_t = match string_format.eq("String") {
                        true => ("str", is_req, None),
                        false => ("fmt", is_req, format_variant),
                    };

                    new_instance.insert(snake_key.to_owned(), (val_t.0, None, is_req));
                    validations.insert(snake_key, val_t);
                },
                "number" => {
                    match is_req {
                        true => {
                            writeln!(out_file, "    pub {}: serde_json::Number,", snake_key).unwrap()
                        },
                        false => {
                            writeln!(out_file, "    #[serde(skip_serializing_if = \"Option::is_none\")]").unwrap();    
                            writeln!(out_file, "    pub {}: Option<serde_json::Number>,", snake_key).unwrap()
                        },
                    }

                    new_instance.insert(snake_key, ("number", None, is_req));
                },
                "boolean" => {
                    match is_req {
                        true => writeln!(out_file, "    pub {}: bool,", snake_key).unwrap(),
                        false => {
                            writeln!(out_file, "    #[serde(skip_serializing_if = \"Option::is_none\")]").unwrap();    
                            writeln!(out_file, "    pub {}: Option<bool>,", snake_key).unwrap()
                        },
                    }
                    
                    new_instance.insert(snake_key, ("boolean", None, is_req));
                },
                "array" => {
                    let item_struct = array_struct_field_writer(k, parent_mod, pk, pv, &snake_key, &mut out_file, &required, &mut impl_buffer);
                    validations.insert(snake_key.to_owned(), ("obj_instance", is_req, None));
                    new_instance.insert(snake_key, ("obj_instance", Some(item_struct), is_req));
                },
                _ => panic!("Unknown type: {}", comp_type)
            }
        }

        // Implement AsMut for component struct
        writeln!(out_file, "}}\n\nimpl AsMut<{}> for {} {{", comp_struct_name, comp_struct_name).unwrap();
        writeln!(out_file, "    fn as_mut(&mut self) -> &mut Self {{").unwrap();
        writeln!(out_file, "        self").unwrap();
        writeln!(out_file, "    }}").unwrap();

        // Add Componentable trait implementation for struct
        writeln!(out_file, "}}\n\nimpl Componentable<{}> for {} {{", comp_struct_name, comp_struct_name).unwrap();
        writeln!(out_file, "    fn validate(&self) -> Result<&Self, UblError> {{").unwrap();
        
        for (pk, (pv, is_req, fmt_var)) in validations {
            match pv {
                "str" => match is_req {
                    true => {
                        writeln!(out_file, "        if self.{}.is_empty() {{", pk).unwrap();
                        writeln!(out_file, "            return Err(UblError::empty(\"{}.{}\"))", k, pk).unwrap();
                        writeln!(out_file, "        }}").unwrap();
                    },
                    false => {
                        writeln!(out_file, "        if let Some(v) = &self.{} {{", pk).unwrap();
                        writeln!(out_file, "            if v.is_empty() {{").unwrap();
                        writeln!(out_file, "                return Err(UblError::optional_empty(\"{}.{}\"))", k, pk).unwrap();
                        writeln!(out_file, "            }}").unwrap();
                        writeln!(out_file, "        }}").unwrap();
                    }
                },
                "fmt" => {
                    let fmt_var = fmt_var.unwrap();

                    match is_req {
                        true => {
                            writeln!(out_file, "        match &self.{} {{", pk).unwrap();
                            writeln!(out_file, "            {} => if s.is_empty() {{", fmt_var).unwrap();
                            writeln!(out_file, "                return Err(UblError::empty(\"{}.{}.{}\"))", k, pk, fmt_var).unwrap();
                            writeln!(out_file, "            }},").unwrap();
                            writeln!(out_file, "            e => return Err(UblError::format(\"{}.{}\", e))", k, pk).unwrap();
                            writeln!(out_file, "        }}").unwrap();
                        },
                        false => {
                            writeln!(out_file, "        if let Some(v) = &self.{} {{", pk).unwrap();
                            writeln!(out_file, "            match v {{").unwrap();
                            writeln!(out_file, "                {} => if s.is_empty() {{", fmt_var).unwrap();
                            writeln!(out_file, "                    return Err(UblError::optional_empty(\"{}.{}.{}\"))", k, pk, fmt_var).unwrap();
                            writeln!(out_file, "                }},").unwrap();
                            writeln!(out_file, "                e => return Err(UblError::optional_format(\"{}.{}\", e))", k, pk).unwrap();
                            writeln!(out_file, "            }}").unwrap();
                            writeln!(out_file, "        }}").unwrap();
                        }
                    }
                },
                "obj_instance" => match is_req {
                    true => {
                        writeln!(out_file, "        if let Err(e) = self.{}.validate() {{", pk).unwrap();
                        writeln!(out_file, "            return Err(UblError::component(\"{}.{}\", e));", k, pk).unwrap();
                        writeln!(out_file, "        }}").unwrap();
                    },
                    false => {
                        writeln!(out_file, "        if let Some(v) = &self.{} {{", pk).unwrap();
                        writeln!(out_file, "            if let Err(e) = v.validate() {{").unwrap();
                        writeln!(out_file, "                return Err(UblError::optional_component(\"{}.{}\", e));", k, pk).unwrap();
                        writeln!(out_file, "            }}").unwrap();
                        writeln!(out_file, "        }}").unwrap();
                    }
                },
                _ => panic!("Unknown validation type: {}", pv)
            }
        }

        writeln!(out_file, "\n        Ok(self)").unwrap();
        writeln!(out_file, "    }}").unwrap();

        writeln!(out_file, "\n    fn get(self) -> Result<Self, UblError> {{").unwrap();
        writeln!(out_file, "        self.validate().map(|s|s.clone())").unwrap();
        writeln!(out_file, "    }}").unwrap();

        writeln!(out_file, "    fn additional_props_allowed() -> bool {{").unwrap();
        writeln!(out_file, "        {}", additional_props).unwrap();
        writeln!(out_file, "    }}").unwrap();

        // Add struct implementation
        writeln!(out_file, "}}\n\nimpl {} {{", comp_struct_name).unwrap();
        impl_title_and_descr_writer(title, description, &mut out_file);

        // Add new instance method, allow strings to come as refereces. Generics from alphabets
        let mut new_instance_args = vec![];
        let mut gen_types = HashMap::new();

        for (pk, (pv, fmt_var, is_req)) in new_instance.iter() {
            if is_req == &false {
                continue;
            }
            
            match pv {
                // Allow generics for required string values only to avoid fn new(argument: None::<&tr>, ...)
                &"str" => match is_req {
                    true => {
                        let index = gen_types.len();
                        
                        match UC_ALPHABETS.get(index) {
                            Some(a) => {
                                gen_types.insert(pk, *a);
                                new_instance_args.push(format!("{}: {}", pk, a));
                            },
                            None => new_instance_args.push(format!("{}: String", pk)),
                        }
                    },
                    false => new_instance_args.push(format!("{}: String", pk)),
                },
                &"fmt" => {
                    new_instance_args.push(format!("{}: crate::FormattedValue", pk));
                },
                &"number" => {
                    new_instance_args.push(format!("{}: serde_json::Number", pk));
                },
                &"boolean" => {
                    new_instance_args.push(format!("{}: bool", pk));
                },
                &"obj_instance" => {
                    new_instance_args.push(format!("{}: {}", pk, fmt_var.clone().unwrap()));
                },
                _ => panic!("Unknown new instance type: {}", pv)
            }
        }

        // Maintain order of arguments
        new_instance_args.sort();

        // Allow string values to come in as a reference
        match gen_types.len() {
            0 => writeln!(out_file, "    pub fn new({}) -> Component<Self> {{", new_instance_args.join(", ")).unwrap(),
            _ => writeln!(
                out_file,
                "    pub fn new<{}>({}) -> Component<Self> where {} {{",
                gen_types.values().into_iter().map(|s|s.to_string()).collect::<Vec<String>>().join(", "),
                new_instance_args.join(", "),
                gen_types.iter().map(|(_, t)|format!("{}: Into<String>", t)).collect::<Vec<String>>().join(", ")
            ).unwrap(),
        }

        writeln!(out_file, "        Component(Self {{").unwrap();

        // For generic string references, convert them to string
        for (pk, (pv, _, is_req)) in new_instance.iter() {
            match pv {
                &"str" => match gen_types.get(pk) {
                    Some(t) => {
                        writeln!(out_file, "            {}: {}.into(), // Generic type: {}", pk, pk, t).unwrap();
                    },
                    // New method ignores optional arguments and inits instance optional fields as None
                    None => match is_req {
                        true => {
                            writeln!(out_file, "            {},", pk).unwrap();
                        },
                        false => {
                            writeln!(out_file, "            {}: None,", pk).unwrap();
                        }
                    }
                },
                _ => match is_req {
                    true => {
                        writeln!(out_file, "            {},", pk).unwrap();
                    },
                    false => {
                        writeln!(out_file, "            {}: None,", pk).unwrap();
                    }
                }
            }
        }

        writeln!(out_file, "        }})").unwrap();
        writeln!(out_file, "    }}").unwrap();
        writeln!(out_file, "}}\n").unwrap();

        for i in impl_buffer {
            writeln!(out_file, "{}", i).unwrap();
        }
    }

    // Delete extensions.rs if its not needed
    if !extensions {
        remove_file(&exts_file_path).ok();
    }

    if mods.len() == 0 {
        remove_dir_all(&comps_dir).ok();

        return formats
    }

    // Create mod.rs file in components directory
    let mut mod_file = comps_dir.clone();
    mod_file.push("mod.rs");

    let mut mod_file = std::fs::File::create(&mod_file)
        .expect("Failed to create file");

    for m in mods.iter() {
        writeln!(mod_file, "mod {};", m).unwrap();
    }

    writeln!(mod_file, "\n").unwrap();

    for m in mods {
        writeln!(mod_file, "pub use {}::*;", m).unwrap();
    }

    formats
}