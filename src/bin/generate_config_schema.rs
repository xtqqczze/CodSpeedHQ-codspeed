//! Generates JSON Schema for codspeed.yaml configuration file
//!
//! Run with:
//! ```
//! cargo run --bin generate-config-schema
//! ```

use std::fs;

use codspeed_runner::ProjectConfig;
use schemars::Schema;
use schemars::generate::SchemaSettings;
use schemars::transform::{Transform, transform_subschemas};

const OUTPUT_FILE: &str = "schemas/codspeed.schema.json";

/// Rewrites `anyOf` to `oneOf` in all schemas (used for untagged enums
/// where variants are mutually exclusive).
#[derive(Clone)]
struct AnyOfToOneOf;

impl Transform for AnyOfToOneOf {
    fn transform(&mut self, schema: &mut Schema) {
        if let Some(any_of) = schema.remove("anyOf") {
            schema.insert("oneOf".to_string(), any_of);
        }
        transform_subschemas(self, schema);
    }
}

fn main() {
    let generator = SchemaSettings::default()
        .with_transform(AnyOfToOneOf)
        .into_generator();
    let schema = generator.into_root_schema_for::<ProjectConfig>();
    let schema_json = serde_json::to_string_pretty(&schema).expect("Failed to serialize schema");
    let output_file_path = std::path::Path::new(OUTPUT_FILE);
    fs::create_dir_all(output_file_path.parent().unwrap())
        .expect("Failed to create schemas directory");
    fs::write(OUTPUT_FILE, format!("{schema_json}\n")).expect("Failed to write schema file");
    println!("Schema written to {OUTPUT_FILE}");
}
