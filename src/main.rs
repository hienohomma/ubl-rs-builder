// mod schema;
mod parser;

use std::io::Write;
use std::path::PathBuf;

use parser::{component_structifier, snaker};


fn main() {
    let in_path = PathBuf::from("./schemas/BDNDR-CCTS_CCT_SchemaModule-1.1.json");
    let mut formats = vec![];
    let mut schemas = vec![];
    let mut post_processables = vec![];
    
    // Create schema lib directory and Cargo.toml file
    let out_path = PathBuf::from("../ubl-rs");
    std::fs::create_dir_all(&out_path)
        .expect("Failed to create ubl-rs library directory");

    let mut cargo_file_path = out_path.to_owned();
    cargo_file_path.push("Cargo.toml");
    
    let mut cargo_file = std::fs::File::create(&cargo_file_path)
        .expect("Failed to create file");

    // Write lib details and dependencies to Cargo.toml file
    writeln!(cargo_file, "[package]
name = \"ubl-rs\"
version = \"0.1.0\"
edition = \"2021\"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
thiserror = {{ version = \"1.0.61\" }}
chrono = {{ version = \"0.4.38\" }}
serde = {{ version = \"1.0.197\", features = [\"derive\"] }}
serde_json = \"1.0.115\"").unwrap();

    // Add readme file to ubl-rs directory
    let mut readme_path = out_path.to_owned();
    readme_path.push("README.md");

    // Add license file to ubl-rs directory
    let mut license_path = out_path.to_owned();
    license_path.push("LICENSE");
    
    // Read license and readme file from ubl-rs-out directory
    let readme_out = PathBuf::from("./ubl-rs-out/README.md");
    let license_out = PathBuf::from("./ubl-rs-out/LICENSE");

    // Copy readme file to ubl-rs directory
    std::fs::copy(&readme_out, &readme_path)
        .expect("Failed to copy ubl-rs README.md file from ./ubl-rs-out directory");

    std::fs::copy(&license_out, &license_path)
        .expect("Failed to copy ubl-rs LICENSE file from ./ubl-rs-out directory");

    // Build and save library
    let out_path = "../ubl-rs/src";
    let t = component_structifier(in_path, out_path);
    schemas.push(t.0);
    formats.extend(t.1);
    post_processables.extend(t.2);

    let in_path = PathBuf::from("./schemas/BDNDR-UnqualifiedDataTypes-1.1.json");
    let t =  component_structifier(in_path, out_path);
    schemas.push(t.0);
    formats.extend(t.1);
    post_processables.extend(t.2);

    let in_path = PathBuf::from("./schemas/UBL-CommonBasicComponents-2.1.json");
    let t =  component_structifier(in_path, out_path);
    schemas.push(t.0);
    formats.extend(t.1);
    post_processables.extend(t.2);

    let in_path = PathBuf::from("./schemas/UBL-CommonAggregateComponents-2.1.json");
    let t =  component_structifier(in_path, out_path);
    schemas.push(t.0);
    formats.extend(t.1);
    post_processables.extend(t.2);

    let in_path = PathBuf::from("./schemas/UBL-CommonExtensionComponents-2.1.json");
    let t =  component_structifier(in_path, out_path);
    schemas.push(t.0);
    formats.extend(t.1);
    post_processables.extend(t.2);

    let in_path = PathBuf::from("./schemas/UBL-QualifiedDataTypes-2.1.json");
    let t =  component_structifier(in_path, out_path);
    schemas.push(t.0);
    formats.extend(t.1);
    post_processables.extend(t.2);

    let in_path = PathBuf::from("./schemas/UBL-ExtensionContentDataType-2.1.json");
    let t =  component_structifier(in_path, out_path);
    schemas.push(t.0);
    formats.extend(t.1);
    post_processables.extend(t.2);

    formats.dedup();

    // Create lib.rs file in ubl-rs directory
    let mut mod_file_path = PathBuf::from(out_path);
    mod_file_path.push("lib.rs");

    let mut mod_file = std::fs::File::create(&mod_file_path)
        .expect("Failed to create file");

    writeln!(mod_file, "pub mod exporter;").unwrap();

    for schema in schemas {
        writeln!(mod_file, "pub mod {};", schema).unwrap();
    }

    writeln!(mod_file, "\n\nuse thiserror::Error;").unwrap();
    writeln!(mod_file, "\n\nuse serde::{{Deserialize, Serialize}};").unwrap();
    writeln!(mod_file, "use std::fmt::{{Display, Formatter, Result as FmtResult}};").unwrap();

    // Add trait and wrapper struct for created components + Error handling
    writeln!(mod_file, "\n\npub trait Componentable<T> {{
    fn validate(&self) -> Result<&T, UblError>;
    fn get(self) -> Result<T, UblError>;
    /// To add additional props to struct, read it as JSON first and manipulate json object props.
    fn additional_props_allowed() -> bool;
}}

pub struct Component<T> (T) where T: Componentable<T> + AsMut<T> + Clone;

impl<T> Component<T> where T: Componentable<T> + AsMut<T> + Clone {{
    pub fn as_mut(&mut self) -> &mut T {{
        self.0.as_mut()
    }}
    pub fn as_validated(&self) -> Result<&T, UblError> {{
        self.0.validate()
    }}
    pub fn get_validated(self) -> Result<T, UblError> {{
        self.0.get()
    }}
}}

#[derive(Error, Debug)]
pub enum UblError {{
    #[error(\"value of `{{0}}` cannot be empty string\")]
    IsEmpty(String),
    #[error(\"value `{{0}}` is optional but when provided cannot be empty string\")]
    OptionalEmpty(String),
    #[error(\"unexpected format for `{{input:?}}`, should not be: {{fmt:?}}\")]
    BadFormat {{
        input: String,
        fmt: String,
    }},
    #[error(\"unable to format input `{{input:?}}` as {{fmt:?}}: {{err:?}}\")]
    InvalidDateTime {{
        input: String,
        fmt: String,
        err: String,
    }},
    #[error(\"value `{{input:?}}` is optional but when provided should not be: {{fmt:?}}\")]
    OptionalBadFormat {{
        input: String,
        fmt: String,
    }},
    #[error(\"component `{{item:?}}` failed validation: {{err:?}}\")]
    ComponentValidation {{
        item: String,
        err: String,
    }},
    #[error(\"optional component `{{item:?}}` failed validation: {{err:?}}\")]
    OptionalComponentValidation {{
        item: String,
        err: String,
    }},
    #[error(\"inner item in component `{{item:?}}` failed validation: {{err:?}}\")]
    InnerComponentValidation {{
        item: String,
        err: String,
    }},
}}

impl UblError {{
    pub fn date_time<I, F>(input: I, fmt: F, err: chrono::ParseError) -> Self where I: Into<String>, F: Into<String> {{
        Self::InvalidDateTime {{
            input: input.into(),
            fmt: fmt.into(),
            err: err.to_string(),
        }}
    }}
    pub fn empty<T>(input: T) -> Self where T: Into<String> {{
        Self::IsEmpty(input.into())
    }}
    pub fn optional_empty<T>(input: T) -> Self where T: Into<String> {{
        Self::OptionalEmpty(input.into())
    }}
    pub fn format<T>(input: T, fmt: &FormattedValue) -> Self where T: Into<String> {{
        Self::BadFormat {{
            input: input.into(),
            fmt: fmt.to_string(),
        }}
    }}
    pub fn optional_format<T>(input: T, fmt: &FormattedValue) -> Self where T: Into<String> {{
        Self::OptionalBadFormat {{
            input: input.into(),
            fmt: fmt.to_string(),
        }}
    }}
    pub fn component<T>(item: T, err: Self) -> Self where T: Into<String> {{
        Self::ComponentValidation {{
            item: item.into(),
            err: err.to_string(),
        }}
    }}
    pub fn optional_component<T>(item: T, err: Self) -> Self where T: Into<String> {{
        Self::OptionalComponentValidation {{
            item: item.into(),
            err: err.to_string(),
        }}
    }}
    pub fn inner_component<T, E>(item: T, err: E) -> Self where T: Into<String>, E: Into<String> {{
        Self::InnerComponentValidation {{
            item: item.into(),
            err: err.into(),
        }}
    }}
}}").unwrap();

    // Add formatted values enum
    writeln!(mod_file, "\n\n#[derive(Debug, Clone, Serialize, Deserialize)]").unwrap();
    writeln!(mod_file, "#[serde(untagged)]").unwrap();
    writeln!(mod_file, "pub enum FormattedValue {{").unwrap();
    
    for f in formats.iter() {
        writeln!(mod_file, "    {},", f).unwrap();
    }

    writeln!(mod_file, "}}").unwrap();

    writeln!(mod_file, "\n\nimpl Display for FormattedValue {{").unwrap();
    writeln!(mod_file, "    fn fmt(&self, f: &mut Formatter) -> FmtResult {{").unwrap();
    writeln!(mod_file, "        match self {{").unwrap();
    
    // Format names are Enums with inner string EnumName(String)
    let format_names = formats.into_iter().map(|s|s.split_once('(').unwrap().0.to_string()).collect::<Vec<String>>();

    for i in format_names.iter() {
        writeln!(mod_file, "            Self::{}(v) => write!(f, \"{{}}\", v),", i).unwrap();
    }

    writeln!(mod_file, "        }}").unwrap();
    writeln!(mod_file, "    }}").unwrap();
    writeln!(mod_file, "}}").unwrap();


    // Implement init method for FormattedValue
    writeln!(mod_file, "\n\nimpl FormattedValue {{").unwrap();

    for i in format_names.iter() {
        // Skip Date, Time, DateTime, since these are handled by chrono
        if ["Date", "Time", "DateTime"].contains(&i.as_str()) {
            continue;
        }

        let snaked = snaker(i);
        writeln!(mod_file, "    pub fn new_{}<T>(v: T) -> Self where T: Into<String> {{", snaked).unwrap();
        writeln!(mod_file, "        Self::{}(v.into())", i).unwrap();
        writeln!(mod_file, "    }}").unwrap();
    }

    // Implement new Date, Time, DateTime methods
    writeln!(mod_file, "    pub fn new_datetime(v: chrono::NaiveDateTime) -> Self {{
        Self::DateTime(v.format(\"%Y-%m-%dT%H:%M:%S\").to_string())
    }}
    /// Create new date time from a string formatted as `YYYY-MM-DD HH:MM:SS`
    pub fn new_date_time_from_str<T>(v: T) -> Result<Self, UblError> where T: AsRef<str> {{
        let s = v.as_ref();
        let fmt = \"%Y-%m-%d %H:%M:%S\";
        chrono::NaiveDateTime::parse_from_str(s, fmt)
            .map(|n|Self::DateTime(n.format(\"%Y-%m-%dT%H:%M:%S\").to_string()))
            .map_err(|e|UblError::date_time(s, fmt, e))
    }}
    /// Create new date time from a string with custom formatting (timezone, fractional seconds, etc.)
    /// Be advised that the formatted value is stored as string in format `YYYY-MM-DDTHH:MM:SS`
    pub fn new_date_time_from_str_in_fmt<T, F>(v: T, format: F) -> Result<Self, UblError> where T: AsRef<str>, F: AsRef<str> {{
        let s = v.as_ref();
        let fmt_in = format.as_ref();
        let fmt_out = \"%Y-%m-%dT%H:%M:%S\";
        chrono::NaiveDateTime::parse_from_str(s, fmt_in)
            .map(|n|Self::DateTime(n.format(fmt_out).to_string()))
            .map_err(|e|UblError::date_time(s, fmt_out, e))
    }}
    pub fn new_date(date: chrono::NaiveDate) -> Self {{
        Self::Date(date.format(\"%Y-%m-%d\").to_string())
    }}
    pub fn new_date_from_str<T>(v: T) -> Result<Self, UblError> where T: AsRef<str> {{
        let s = v.as_ref();
        let fmt = \"%Y-%m-%d\";
        chrono::NaiveDate::parse_from_str(s, fmt)
            .map(|n|Self::Date(n.format(fmt).to_string()))
            .map_err(|e|UblError::date_time(s, fmt, e))
    }}
    pub fn new_date_from_str_in_fmt<T, F>(v: T, format: F) -> Result<Self, UblError> where T: AsRef<str>, F: AsRef<str> {{
        let s = v.as_ref();
        let fmt_in = format.as_ref();
        let fmt_out = \"%Y-%m-%d\";
        chrono::NaiveDate::parse_from_str(s, fmt_in)
            .map(|n|Self::Date(n.format(fmt_out).to_string()))
            .map_err(|e|UblError::date_time(s, fmt_out, e))
    }}
    pub fn new_time(time: chrono::NaiveTime) -> Self {{
        Self::Time(time.format(\"%H:%M:%S\").to_string())
    }}
    pub fn new_time_from_str<T>(v: T) -> Result<Self, UblError> where T: AsRef<str> {{
        let s = v.as_ref();
        let fmt = \"%H:%M:%S\";
        chrono::NaiveTime::parse_from_str(s, fmt)
            .map(|n|Self::Time(n.format(fmt).to_string()))
            .map_err(|e|UblError::date_time(s, fmt, e))
    }}
    pub fn new_time_from_str_in_fmt<T, F>(v: T, format: F) -> Result<Self, UblError> where T: AsRef<str>, F: AsRef<str> {{
        let s = v.as_ref();
        let fmt_in = format.as_ref();
        let fmt_out = \"%H:%M:%S\";
        chrono::NaiveTime::parse_from_str(s, fmt_in)
            .map(|n|Self::Time(n.format(fmt_out).to_string()))
            .map_err(|e|UblError::date_time(s, fmt_out, e))
    }}").unwrap();

    writeln!(mod_file, "}}").unwrap();


    for (refs_file, (_referenced_path, comp_ref)) in post_processables {
        let mut out_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&refs_file)
            .expect("Failed to open mod reference file");

        // println!("comp ref > [{}]: {:?}", referenced_path.exists(), referenced_path);

        writeln!(out_file, "{}",comp_ref).unwrap();
    }

    // Check if exporter.rs file is present in schema/src directory
    let exporter_rs = PathBuf::from("../ubl-rs/src/exporter.rs");
    
    // Create empty file if missing
    if !exporter_rs.exists() {
        std::fs::File::create(&exporter_rs)
            .expect("Failed to create exporter.rs file to schema lib");
    }
}
