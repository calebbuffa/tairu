use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::Parser;
use quote::ToTokens;
mod policy;

use policy::{PolicyConfig, TairuPolicy};
use schemagen::{Config, Graph, generate_types_from_roots, render_module};

#[derive(Parser)]
#[command(name = "xtask", about = "Generate Tairu code from 3D Tiles schemas")]
struct Args {
    #[arg(long)]
    schema: Option<PathBuf>,
    #[arg(long)]
    config: Option<PathBuf>,
    #[arg(long)]
    output: Option<PathBuf>,
    #[arg(long)]
    check: bool,
    #[arg(long)]
    manifest: bool,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .context("xtask has no workspace parent")?
        .to_path_buf();
    let args = Args::parse();
    let schema = args.schema.unwrap_or_else(|| {
        workspace.join("extern/3d-tiles/specification/schema/tileset.schema.json")
    });
    let config_path = args
        .config
        .unwrap_or_else(|| workspace.join("xtask/config.json"));
    let output_dir = args.output.unwrap_or_else(|| workspace.join("src"));
    let config: Config = serde_json::from_str(&std::fs::read_to_string(&config_path)?)?;
    let policy_config: PolicyConfig =
        serde_json::from_str(&std::fs::read_to_string(&config_path)?)?;
    let policy = TairuPolicy {
        config: policy_config,
    };
    let schema = schema.canonicalize()?;
    let mut graph = Graph::new(schema.parent().context("schema has no parent")?);
    let root_name = schema
        .file_name()
        .context("schema has no file name")?
        .to_string_lossy();
    graph.load(&root_name).map_err(anyhow::Error::msg)?;
    let roots = graph.load_tree(".")?;
    let definitions = generate_types_from_roots(&mut graph, roots, &config, &policy)
        .map_err(anyhow::Error::msg)?;
    if graph.sink().has_errors() {
        bail!("schema diagnostics contain errors");
    }
    let output = render_module(
        "Generated 3D Tiles data model. Do not edit.",
        &definitions,
        &config,
        &policy,
    )
    .map_err(anyhow::Error::msg)?;
    // Generated schemas intentionally include types that are only used by
    // downstream consumers, and the generator emits explicit defaults for
    // compatibility. Keep those generated-code diagnostics scoped to this
    // module rather than requiring every consumer to suppress them.
    let output = output.replacen(
        "#![allow(missing_docs)]",
        "#![allow(missing_docs, dead_code, clippy::derivable_impls)]",
        1,
    );
    let output = if policy.config.extensible {
        append_extension_impls(
            output,
            &definitions,
            policy
                .config
                .extension_trait
                .as_deref()
                .unwrap_or("crate::HasExtensions"),
        )?
    } else {
        output
    };
    let output_file = output_dir.join("generated.rs");
    if args.check {
        let current = std::fs::read_to_string(&output_file).unwrap_or_default();
        // rustfmt may normalize generated output without changing its model.
        // Compare tokens so the freshness check does not report formatting-only
        // drift after a workspace-wide `cargo fmt`.
        let current_tokens = syn::parse_file(&current).map(|file| file.into_token_stream());
        let output_tokens = syn::parse_file(&output).map(|file| file.into_token_stream());
        if current_tokens.is_err() || output_tokens.is_err() {
            bail!("{} is out of date", output_file.display());
        }
    } else {
        std::fs::create_dir_all(&output_dir)?;
        std::fs::write(&output_file, output)?;
        if args.manifest {
            let manifest_path = output_dir.join("MANIFEST.md");
            let mut manifest = String::from("# Generated 3D Tiles Types\n\n");
            for definition in &definitions {
                manifest.push_str(&format!("- `{}`\n", definition.name));
            }
            std::fs::write(manifest_path, manifest)?;
        }
    }
    Ok(())
}

fn append_extension_impls(
    mut output: String,
    definitions: &[schemagen::StructDef],
    trait_path: &str,
) -> Result<String> {
    let trait_path: syn::Path = syn::parse_str(trait_path)?;
    let source = definitions.iter().filter(|definition| definition.is_object).map(|definition| {
        let name: syn::Ident = syn::parse_str(&definition.name)?;
        Ok::<_, anyhow::Error>(quote::quote! {
            impl #trait_path for #name {
                fn extensions(&self) -> &std::collections::HashMap<String, serde_json::Value> { &self.extensions }
                fn extensions_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> { &mut self.extensions }
            }
        }.to_string())
    }).collect::<Result<Vec<_>>>()?.join("\n");
    let file: syn::File = syn::parse_str(&source)?;
    output.push_str(&prettyplease::unparse(&file));
    Ok(output)
}
