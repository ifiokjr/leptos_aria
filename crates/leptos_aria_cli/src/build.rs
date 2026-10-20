//! `leptos-aria build`: turn a `registry.json` source manifest into the
//! static files a registry serves, exactly as `shadcn build` does. Each item
//! becomes `<output>/<name>.json` with its file contents inlined, and the
//! index is copied to `<output>/registry.json` without contents for `list`.

use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use serde::Serialize;

use crate::registry::ITEM_SCHEMA;
use crate::registry::REGISTRY_SCHEMA;
use crate::registry::Registry;
use crate::registry::RegistryItem;

/// What `build` wrote, and the dependencies it left to the official registry.
#[derive(Debug)]
pub struct Built {
	/// Every file written, the index last.
	pub written: Vec<PathBuf>,
	/// Bare `registryDependencies` this registry does not define. They resolve
	/// against the official registry at install time, like shadcn's bare names.
	pub external: Vec<String>,
}

/// Build every item in `manifest` into `output`.
pub fn build(manifest: &Path, output: &Path) -> Result<Built> {
	let text =
		fs::read_to_string(manifest).with_context(|| format!("reading {}", manifest.display()))?;
	let mut registry: Registry =
		serde_json::from_str(&text).with_context(|| format!("parsing {}", manifest.display()))?;
	let base = manifest.parent().unwrap_or_else(|| Path::new("."));

	check_names(&registry)?;
	let external = external_dependencies(&registry);
	fs::create_dir_all(output).with_context(|| format!("creating {}", output.display()))?;

	let mut written = Vec::with_capacity(registry.items.len() + 1);

	for item in &registry.items {
		let path = output.join(format!("{}.json", item.name));
		write_json(&path, &inline_files(item, base)?)?;
		written.push(path);
	}

	registry.schema = Some(REGISTRY_SCHEMA.to_string());
	let index = output.join("registry.json");
	write_json(&index, &registry)?;
	written.push(index);

	Ok(Built { written, external })
}

/// A copy of the item with its schema set and every file's contents read
/// from disk, relative to the manifest.
fn inline_files(item: &RegistryItem, base: &Path) -> Result<RegistryItem> {
	let mut built = item.clone();
	built.schema = Some(ITEM_SCHEMA.to_string());

	for file in &mut built.files {
		let source = base.join(&file.path);
		let content = fs::read_to_string(&source)
			.with_context(|| format!("reading {} for `{}`", source.display(), item.name))?;
		file.content = Some(content);
	}

	Ok(built)
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<()> {
	let json = serde_json::to_string_pretty(value)?;
	fs::write(path, format!("{json}\n")).with_context(|| format!("writing {}", path.display()))
}

/// Reject duplicate names and the `registry` name, which the index uses.
fn check_names(registry: &Registry) -> Result<()> {
	let mut names = HashSet::new();

	for item in &registry.items {
		if item.name == "registry" {
			bail!("`registry` is reserved for the index; rename that item");
		}

		if !names.insert(item.name.as_str()) {
			bail!("two items are named `{}`", item.name);
		}
	}

	Ok(())
}

/// Bare `registryDependencies` that name no item in this registry, as
/// `item -> dependency`.
fn external_dependencies(registry: &Registry) -> Vec<String> {
	let names: HashSet<&str> = registry
		.items
		.iter()
		.map(|item| item.name.as_str())
		.collect();

	registry
		.items
		.iter()
		.flat_map(|item| {
			item.registry_dependencies
				.iter()
				.map(move |dependency| (item, dependency))
		})
		.filter(|(_, dependency)| is_bare(dependency) && !names.contains(dependency.as_str()))
		.map(|(item, dependency)| format!("{} -> {dependency}", item.name))
		.collect()
}

/// Whether a dependency is a plain item name rather than a namespace, URL, or
/// `.json` path.
fn is_bare(dependency: &str) -> bool {
	let is_json = Path::new(dependency)
		.extension()
		.is_some_and(|extension| extension == "json");

	!(dependency.starts_with('@') || dependency.contains("://") || is_json)
}

#[cfg(test)]
mod tests {
	use super::*;

	fn registry(json: &str) -> Registry {
		serde_json::from_str(json).unwrap()
	}

	#[test]
	fn rejects_duplicates_and_reports_external_dependencies() {
		let missing = registry(
			r#"{"name":"r","items":[{"name":"button","type":"registry:ui","registryDependencies":["utils"]}]}"#,
		);
		assert_eq!(external_dependencies(&missing), ["button -> utils"]);

		let duplicate = registry(
			r#"{"name":"r","items":[{"name":"a","type":"registry:ui"},{"name":"a","type":"registry:ui"}]}"#,
		);
		assert!(check_names(&duplicate).is_err());

		let external = registry(
			r#"{"name":"r","items":[{"name":"a","type":"registry:ui","registryDependencies":["@acme/b","https://x.dev/r/c.json"]}]}"#,
		);
		assert_eq!(external_dependencies(&external), Vec::<String>::new());
		assert!(
			check_names(&registry(
				r#"{"name":"r","items":[{"name":"registry","type":"registry:ui"}]}"#
			))
			.is_err()
		);
	}
}
