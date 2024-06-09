use serde_json::Value;
use std::fs::File;
use std::io::Write;

use super::reference::ref_path_splitter;
use super::struct_namer;

// Array references have "items" object with "$ref" key. Parent object has "type" key with value "array"
// and optional title and description.
// Example of array reference:
// "SupplyChainActivityTypeCode": {
//     "title": "Activity Data Line. Supply Chain Activity Type Code. Code",
//     "description": "A code signifying the type of supply chain activity.",
//     "items": {
//       "$ref": "UBL-CommonBasicComponents-2.1.json#/definitions/SupplyChainActivityTypeCode"
//       },
//     "maxItems": 1,
//     "minItems": 1,
//     "type": "array"
//     },

pub fn array_struct_field_writer(parent_key: &String, parent_mod: &str, key: &str,
val: &Value, snake_key: &String, out_file: &mut File, required: &Vec<&str>,
impl_buffer: &mut Vec<String>) -> String {
    let item_key = format!("{}ArrayOf{}Component", parent_key, key);
    let item_struct = struct_namer(item_key);

    match required.contains(&key) {
        true => {
            writeln!(out_file, "    pub {}: {},", snake_key, item_struct).unwrap()
        },
        false => {
            writeln!(out_file, "    #[serde(skip_serializing_if = \"Option::is_none\")]").unwrap();    
            writeln!(out_file, "    pub {}: Option<{}>,", snake_key, item_struct).unwrap()
        },
    }

    let items = val.get("items")
        .expect("No items in array type component")
        .as_object()
        .expect("Items is not an object");

    // TODO: is this always the case? Items is always a reference to a single component?
    if items.len() != 1 {
        panic!("Items is not an object with one key. Changed since last spec. Can items hold more than one reference?");
    }

    let ref_path = items.get("$ref")
        .expect("No reference in items")
        .as_str()
        .expect("Reference is not a string");

    let (subdir, _, ref_struct_name) = ref_path_splitter(ref_path);

    // Add struct definition
    let mut vals = vec![String::from("#[derive(Debug, Clone, Deserialize, Serialize)]")];
    // Items should not be displayed on the serialized json
    vals.push(format!("#[serde(transparent)]\npub struct {} {{", item_struct));

    // Determine path to referenced struct
    let ref_mod_trail = match subdir {
        Some(s) => format!("crate::{}::{}", s, ref_struct_name),
        None => format!("crate::{}::{}", parent_mod, ref_struct_name),
    };
    
    vals.push(format!("    pub items: Vec<{}>,", ref_mod_trail));

    // Implement AsMut for component struct
    vals.push(format!("}}\n\nimpl AsMut<{}> for {} {{", item_struct, item_struct));
    vals.push(String::from("    fn as_mut(&mut self) -> &mut Self {"));
    vals.push(String::from("        self"));
    vals.push(String::from("    }"));

    // Add Componentable trait implementation for struct
    vals.push(format!("}}\n\nimpl Componentable<{}> for {} {{", item_struct, item_struct));
    vals.push(String::from("    fn validate(&self) -> Result<&Self, UblError> {"));

    if val.get("maxItems").is_some() || val.get("minItems").is_some() {
        vals.push(format!("        let len = self.items.len();\n"));
    }

    if let Some(v) = val.get("maxItems") {
        let u = v.as_u64().expect("maxItems is not a number");

        vals.push(format!("        if len > {} {{", u));
        vals.push(format!("            return Err(UblError::inner_component(\"{}\", format!(\"Max allowed items is {} and you have {{}}\", len)))", item_struct, u));
        vals.push(format!("        }}"));
    }
    
    if let Some(v) = val.get("minItems") {
        let u = v.as_u64().expect("minItems is not a number");

        vals.push(format!("        if len < {} {{", u));
        vals.push(format!("            return Err(UblError::inner_component(\"{}\", format!(\"Min allowed items is {} and you have {{}}\", len)))", item_struct, u));
        vals.push(format!("        }}"));
    }

    vals.push(format!("\n        Ok(self)"));
    vals.push(format!("    }}"));

    vals.push(String::from("    fn get(self) -> Result<Self, UblError> {"));
    vals.push(String::from("        self.validate().map(|s|s.clone())"));
    vals.push(String::from("    }"));

    vals.push(String::from("    fn additional_props_allowed() -> bool {"));
    vals.push(String::from("        false"));
    vals.push(String::from("    }"));

    // Add struct implementation
    vals.push(format!("}}\n\nimpl {} {{", item_struct));

    // Methods for: initializing new struct with one item
    vals.push(format!("    pub fn new(item: {}) -> Component<Self> {{", ref_mod_trail));
    vals.push(format!("        Component(Self {{
            items: vec![item],
        }})
    }}"));

    // ..pushing new item to existing struct
    vals.push(format!("    pub fn push(&mut self, item: {}) {{", ref_mod_trail));
    vals.push(format!("        self.items.push(item);
    }}"));

    // iterating mutably over added items
    vals.push(format!("    pub fn iter_mut(&mut self) -> std::slice::IterMut<{}> {{", ref_mod_trail));
    vals.push(format!("        self.items.iter_mut()
    }}"));

    // iterating over added items
    vals.push(format!("    pub fn iter(&self) -> std::slice::Iter<{}> {{", ref_mod_trail));
    vals.push(format!("        self.items.iter()
    }}\n}}\n"));

    impl_buffer.extend(vals);

    item_struct
}
