//! Registry items and where to fetch them.
//!
//! Items follow shadcn/ui's `registry-item.json` schema, so the same
//! vocabulary applies: `registryDependencies` name other items,
//! `dependencies` name crates for `cargo add`, `cssVars` and `css` extend the
//! Tailwind CSS file. An item is addressed by bare name (the default
//! registry), `@namespace/name` (a registry from `leptos-aria.json`), a URL,
//! or a local `.json` path.

use std::env;
use std::fs;
use std::path::Path;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use indexmap::IndexMap;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Map;
use serde_json::Value;

use crate::config::Config;

/// The `$schema` of a built registry item.
pub const ITEM_SCHEMA: &str = "https://ui.shadcn.com/schema/registry-item.json";

/// The `$schema` of a registry index.
pub const REGISTRY_SCHEMA: &str = "https://ui.shadcn.com/schema/registry.json";

/// The official registry, as a `{name}` template.
pub const DEFAULT_REGISTRY: &str = "https://ifiokjr.github.io/leptos_aria/r/{name}.json";

/// Overrides [`DEFAULT_REGISTRY`], for example to install from a local build:
/// `LEPTOS_ARIA_REGISTRY_URL=./public/r/{name}.json`.
pub const REGISTRY_ENV: &str = "LEPTOS_ARIA_REGISTRY_URL";

/// A registry index: `registry.json`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Registry {
	/// Editor validation.
	#[serde(rename = "$schema", default, skip_serializing_if = "Option::is_none")]
	pub schema: Option<String>,
	/// The registry name.
	pub name: String,
	/// Where the registry is documented.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub homepage: Option<String>,
	/// Every item the registry serves.
	pub items: Vec<RegistryItem>,
}

/// One installable item.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RegistryItem {
	/// Editor validation.
	#[serde(rename = "$schema", default, skip_serializing_if = "Option::is_none")]
	pub schema: Option<String>,
	/// Unique name within the registry; also the `add` argument.
	pub name: String,
	/// The item kind, such as `registry:ui`, `registry:lib`, or `registry:style`.
	#[serde(rename = "type")]
	pub kind: String,
	/// Short human-readable label.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub title: Option<String>,
	/// One-sentence summary.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,
	/// Crates the item needs, as `cargo add` arguments (`name` or `name@version`).
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub dependencies: Vec<String>,
	/// Other items installed first.
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub registry_dependencies: Vec<String>,
	/// The files the item copies.
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub files: Vec<RegistryFile>,
	/// CSS variables for the `theme` (`@theme inline`), `light` (`:root`), and
	/// `dark` (`.dark`) scopes.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub css_vars: Option<CssVars>,
	/// Extra CSS as nested rules: `{"@layer base": {"body": {"@apply bg-background": {}}}}`.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub css: Option<Map<String, Value>>,
	/// A message printed after installing.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub docs: Option<String>,
	/// Labels for grouping.
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub categories: Vec<String>,
}

/// A file inside an item.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct RegistryFile {
	/// The source path within the registry.
	pub path: String,
	/// The file kind, such as `registry:ui`.
	#[serde(rename = "type")]
	pub kind: String,
	/// Where to write it. `@ui/` is the configured component directory and
	/// `~/` the project root; without a target the file lands in `@ui/`.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub target: Option<String>,
	/// The file contents, inlined by `leptos-aria build`.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub content: Option<String>,
}

/// CSS variables grouped by scope, in declaration order.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct CssVars {
	/// Written inside `@theme inline`.
	#[serde(default, skip_serializing_if = "IndexMap::is_empty")]
	pub theme: IndexMap<String, String>,
	/// Written inside `:root`.
	#[serde(default, skip_serializing_if = "IndexMap::is_empty")]
	pub light: IndexMap<String, String>,
	/// Written inside `.dark`.
	#[serde(default, skip_serializing_if = "IndexMap::is_empty")]
	pub dark: IndexMap<String, String>,
}

/// Where an item or index is read from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Source {
	/// An `http(s)` URL.
	Url(String),
	/// A path on disk.
	File(String),
}

impl Source {
	/// Classify a URL or path.
	fn from_location(location: String) -> Self {
		if location.starts_with("http://") || location.starts_with("https://") {
			Self::Url(location)
		} else {
			Self::File(location)
		}
	}

	/// Read the source as text.
	pub fn read(&self) -> Result<String> {
		match self {
			Self::Url(url) => {
				ureq::get(url)
					.call()
					.with_context(|| format!("fetching {url}"))?
					.body_mut()
					.read_to_string()
					.with_context(|| format!("reading {url}"))
			}
			Self::File(path) => fs::read_to_string(path).with_context(|| format!("reading {path}")),
		}
	}
}

impl std::fmt::Display for Source {
	fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Url(location) | Self::File(location) => formatter.write_str(location),
		}
	}
}

/// The default registry template: [`REGISTRY_ENV`] or [`DEFAULT_REGISTRY`].
pub fn default_template() -> String {
	env::var(REGISTRY_ENV)
		.ok()
		.filter(|value| !value.is_empty())
		.unwrap_or_else(|| DEFAULT_REGISTRY.to_string())
}

/// Resolve an item address to the place its JSON is read from.
pub fn resolve(address: &str, config: Option<&Config>, root: &Path) -> Result<Source> {
	if address.starts_with("http://") || address.starts_with("https://") {
		return Ok(Source::Url(address.to_string()));
	}

	if Path::new(address)
		.extension()
		.is_some_and(|extension| extension == "json")
	{
		let path = root.join(address);
		return Ok(Source::File(path.to_string_lossy().into_owned()));
	}

	let (template, name) = match address.strip_prefix('@') {
		Some(rest) => {
			let Some((namespace, name)) = rest.split_once('/') else {
				bail!("`{address}` needs a name after the namespace, like `@acme/button`");
			};
			let namespace = format!("@{namespace}");
			let Some(template) = config.and_then(|config| config.registries.get(&namespace)) else {
				bail!(
					"unknown registry `{namespace}`. Add it to `registries` in leptos-aria.json: \
					 \"{namespace}\": \"https://example.com/r/{{name}}.json\""
				);
			};

			(template.clone(), name)
		}
		None => (default_template(), address),
	};

	Ok(Source::from_location(template.replace("{name}", name)))
}

/// The index (`registry.json`) beside the items a template points at.
pub fn index_source(namespace: Option<&str>, config: Option<&Config>) -> Result<Source> {
	let template = match namespace {
		Some(namespace) => {
			config
				.and_then(|config| config.registries.get(namespace))
				.cloned()
				.with_context(|| format!("unknown registry `{namespace}`"))?
		}
		None => default_template(),
	};

	Ok(Source::from_location(
		template.replace("{name}", "registry"),
	))
}

/// Fetch and parse one item.
pub fn fetch_item(source: &Source) -> Result<RegistryItem> {
	let text = source.read()?;
	serde_json::from_str(&text).with_context(|| format!("parsing the registry item at {source}"))
}

/// Fetch and parse a registry index.
pub fn fetch_index(source: &Source) -> Result<Registry> {
	let text = source.read()?;
	serde_json::from_str(&text).with_context(|| format!("parsing the registry at {source}"))
}

/// Fetch the named items and everything they depend on, dependencies first,
/// each item once.
pub fn fetch_tree(
	addresses: &[String],
	config: Option<&Config>,
	root: &Path,
) -> Result<Vec<RegistryItem>> {
	let mut ordered: Vec<RegistryItem> = Vec::new();
	let mut visiting: Vec<String> = Vec::new();

	for address in addresses {
		visit(address, config, root, &mut ordered, &mut visiting)?;
	}

	Ok(ordered)
}

fn visit(
	address: &str,
	config: Option<&Config>,
	root: &Path,
	ordered: &mut Vec<RegistryItem>,
	visiting: &mut Vec<String>,
) -> Result<()> {
	let source = resolve(address, config, root)?;
	let key = source.to_string();

	if visiting.contains(&key) {
		return Ok(());
	}

	visiting.push(key);

	let item = fetch_item(&source)?;

	if ordered.iter().any(|existing| existing.name == item.name) {
		return Ok(());
	}

	for dependency in &item.registry_dependencies {
		visit(dependency, config, root, ordered, visiting)?;
	}

	ordered.push(item);
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	fn config_with_acme() -> Config {
		let mut config = Config::new("style/tailwind.css", "src/components/ui");
		config.registries.insert(
			"@acme".to_string(),
			"https://acme.dev/r/{name}.json".to_string(),
		);
		config
	}

	#[test]
	fn resolves_every_address_form() {
		let config = config_with_acme();
		let root = Path::new("/project");

		assert_eq!(
			resolve("@acme/card", Some(&config), root).unwrap(),
			Source::Url("https://acme.dev/r/card.json".to_string())
		);
		assert_eq!(
			resolve("https://x.dev/r/a.json", None, root).unwrap(),
			Source::Url("https://x.dev/r/a.json".to_string())
		);
		assert_eq!(
			resolve("items/a.json", None, root).unwrap(),
			Source::File("/project/items/a.json".to_string())
		);
		assert!(resolve("@missing/card", Some(&config), root).is_err());
		assert!(resolve("@acme", Some(&config), root).is_err());
	}

	#[test]
	fn derives_the_index_beside_the_items() {
		let config = config_with_acme();

		assert_eq!(
			index_source(Some("@acme"), Some(&config)).unwrap(),
			Source::Url("https://acme.dev/r/registry.json".to_string())
		);
	}

	#[test]
	fn parses_a_shadcn_item() {
		let item: RegistryItem = serde_json::from_str(
			r#"{
				"name": "button",
				"type": "registry:ui",
				"registryDependencies": ["utils"],
				"files": [{"path": "src/ui/button.rs", "type": "registry:ui", "content": "x"}],
				"cssVars": {"light": {"brand": "red"}}
			}"#,
		)
		.unwrap();

		assert_eq!(item.registry_dependencies, ["utils"]);
		assert_eq!(item.files[0].content.as_deref(), Some("x"));
		assert_eq!(item.css_vars.unwrap().light["brand"], "red");
	}
}
