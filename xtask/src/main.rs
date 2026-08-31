mod config;
mod generate;
mod schema;

use std::path::PathBuf;

use clap::Parser;
use std::collections::{HashSet, VecDeque};

use config::Config;
use generate::{GeneratedStruct, generate_struct, render_manifest, render_module};
use schema::SchemaCache;

#[derive(Parser)]
#[command(
    name = "xtask",
    about = "Regenerate tairu/src/generated.rs from 3D Tiles JSON Schema"
)]
struct Args {
    /// Override the root schema file (defaults to extern/3d-tiles/specification/schema/tileset.schema.json).
    #[arg(long)]
    schema: Option<PathBuf>,

    /// Override the config file (defaults to xtask/config.json).
    #[arg(long)]
    config: Option<PathBuf>,

    /// Override the output directory (defaults to tairu/src/).
    #[arg(long)]
    output: Option<PathBuf>,

    /// Check mode: exit non-zero if generated output would change.
    #[arg(long)]
    check: bool,
}

fn main() {
    // CARGO_MANIFEST_DIR is the xtask/ directory; workspace root is one level up.
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();

    let args = Args::parse();

    let schema_file = args.schema.unwrap_or_else(|| {
        workspace_root
            .join("extern")
            .join("3d-tiles")
            .join("specification")
            .join("schema")
            .join("tileset.schema.json")
    });

    let config_file = args
        .config
        .unwrap_or_else(|| workspace_root.join("xtask").join("config.json"));

    let output_dir = args
        .output
        .unwrap_or_else(|| workspace_root.join("tairu").join("src"));

    let schema_dir = schema_file.parent().unwrap().to_path_buf();

    println!("Regenerating 3D Tiles types...");
    println!("  Schema: {}", schema_file.display());
    println!("  Config: {}", config_file.display());
    println!("  Output: {}", output_dir.display());

    let config_text = std::fs::read_to_string(&config_file)
        .unwrap_or_else(|e| panic!("Cannot read config {}: {e}", config_file.display()));
    let config: Config =
        serde_json::from_str(&config_text).unwrap_or_else(|e| panic!("Cannot parse config: {e}"));

    let mut cache = SchemaCache::new(vec![schema_dir.clone()]);
    let (root_schema, root_path) = cache
        .load_with_path(schema_file.to_string_lossy().as_ref())
        .unwrap_or_else(|| panic!("Cannot load schema {}", schema_file.display()));

    let mut queue: VecDeque<(String, schema::JsonSchema, Option<PathBuf>)> = VecDeque::new();
    let mut seen: HashSet<String> = HashSet::new();
    let mut structs: Vec<GeneratedStruct> = Vec::new();
    let mut custom_types = config.custom_types.clone();

    queue.push_back((
        root_schema.title.clone().unwrap_or_default(),
        root_schema.clone(),
        root_path,
    ));

    for schema_name in &config.additional_schemas {
        if let Some((schema, path)) = cache.load_with_path(schema_name) {
            queue.push_back((schema.title.clone().unwrap_or_default(), schema, path));
        }
    }

    while let Some((title, schema, source_path)) = queue.pop_front() {
        if seen.contains(&title) {
            continue;
        }
        seen.insert(title.clone());

        if let Some(generated) = generate_struct(
            &mut cache,
            &config,
            &mut custom_types,
            &schema,
            source_path.as_deref(),
        ) {
            for field in &generated.fields {
                for discovered in &field.discovered_schemas {
                    if !seen.contains(&discovered.title) {
                        queue.push_back((
                            discovered.title.clone(),
                            discovered.schema.clone(),
                            discovered.source_path.clone(),
                        ));
                    }
                }
            }
            structs.push(generated);
        }
    }

    let module_doc = "Generated 3D Tiles 1.0/1.1 data model. Do not edit — run `cargo run -p xtask` to regenerate.";
    let mut output = render_module(module_doc, &structs, &[], &custom_types);
    output.push_str(&render_generated_extension_impls_from_module(&output));

    let out_file = output_dir.join("generated.rs");

    if args.check {
        let existing = std::fs::read_to_string(&out_file).unwrap_or_default();
        if existing != output {
            eprintln!("generated.rs is out of date — run `cargo run -p xtask` to update.");
            std::process::exit(1);
        }
    } else {
        std::fs::create_dir_all(&output_dir).expect("create output dir");
        std::fs::write(&out_file, &output).unwrap_or_else(|e| panic!("write: {e}"));
        println!("Wrote {} structs to {}", structs.len(), out_file.display());
        let manifest = render_manifest(&structs, &custom_types, &config);
        let manifest_path = output_dir.join("..").join("MANIFEST.md");
        std::fs::write(&manifest_path, manifest).ok();
    }
}

fn render_generated_extension_impls_from_module(module_src: &str) -> String {
    let mut names: Vec<String> = Vec::new();
    let mut current_struct: Option<String> = None;
    for line in module_src.lines() {
        let trimmed = line.trim();
        if let Some(name_part) = trimmed.strip_prefix("pub struct ") {
            if let Some((name, _)) = name_part.split_once(' ') {
                current_struct = Some(name.trim().to_string());
            }
            continue;
        }
        if trimmed
            .starts_with("pub extensions: std::collections::HashMap<String, serde_json::Value>,")
        {
            if let Some(name) = &current_struct {
                names.push(name.clone());
            }
        }
    }
    names.sort();
    names.dedup();

    let mut out = String::new();
    out.push_str(
        "\n// ---------------------------------------------------------------------------\n",
    );
    out.push_str("// Generated typed extension bindings\n");
    out.push_str(
        "// ---------------------------------------------------------------------------\n\n",
    );
    for name in names {
        out.push_str("impl crate::HasExtensions for ");
        out.push_str(&name);
        out.push_str(" {\n");
        out.push_str(
            "    fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> {\n",
        );
        out.push_str("        &self.extensions\n");
        out.push_str("    }\n\n");
        out.push_str(
            "    fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {\n",
        );
        out.push_str("        &mut self.extensions\n");
        out.push_str("    }\n");
        out.push_str("}\n\n");
    }
    out
}
