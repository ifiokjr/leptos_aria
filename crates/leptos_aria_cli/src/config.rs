//! `leptos-aria.json`: the project configuration `init` writes and every
//! other command reads, the counterpart of shadcn/ui's `components.json`.

use std::fs;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use indexmap::IndexMap;
use serde::Deserialize;
use serde::Serialize;

/// The configuration file name, read from the project root.
pub const CONFIG_FILE: &str = "leptos-aria.json";

/// The JSON schema published with the website.
pub const CONFIG_SCHEMA: &str = "https://ifiokjr.github.io/leptos_aria/schema.json";

/// The project configuration.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Config {
	/// Editor validation for this file.
	#[serde(rename = "$schema", default = "default_schema")]
	pub schema: String,
	/// The registry style installed by `init`.
	#[serde(default = "default_style")]
	pub style: String,
	/// Where Tailwind lives.
	pub tailwind: TailwindConfig,
	/// Where copied files land.
	pub paths: PathsConfig,
	/// Extra registries, keyed by `@namespace`, each a URL or path template
	/// containing `{name}`: `"@acme": "https://acme.dev/r/{name}.json"`.
	#[serde(default, skip_serializing_if = "IndexMap::is_empty")]
	pub registries: IndexMap<String, String>,
}

/// Tailwind settings.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct TailwindConfig {
	/// The CSS file that contains `@import "tailwindcss"`. Theme variables and
	/// component CSS are written here.
	pub css: String,
}

/// Install locations, relative to the project root.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PathsConfig {
	/// The module directory that receives components, `src/components/ui`
	/// by default. Its `mod.rs` gains a `pub mod` line per component.
	pub ui: String,
}

impl Config {
	/// A configuration with the given CSS file and component directory.
	pub fn new(css: impl Into<String>, ui: impl Into<String>) -> Self {
		Self {
			schema: default_schema(),
			style: default_style(),
			tailwind: TailwindConfig { css: css.into() },
			paths: PathsConfig { ui: ui.into() },
			registries: IndexMap::new(),
		}
	}

	/// Read `leptos-aria.json` from the project root.
	pub fn load(root: &Path) -> Result<Self> {
		let path = root.join(CONFIG_FILE);

		if !path.exists() {
			bail!(
				"{} not found in {}. Run `leptos-aria init` first.",
				CONFIG_FILE,
				root.display()
			);
		}

		let text =
			fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
		serde_json::from_str(&text).with_context(|| format!("parsing {}", path.display()))
	}

	/// Write `leptos-aria.json` into the project root.
	pub fn save(&self, root: &Path) -> Result<PathBuf> {
		let path = root.join(CONFIG_FILE);
		let json = serde_json::to_string_pretty(self)?;

		fs::write(&path, format!("{json}\n"))
			.with_context(|| format!("writing {}", path.display()))?;
		Ok(path)
	}
}

fn default_schema() -> String {
	CONFIG_SCHEMA.to_string()
}

fn default_style() -> String {
	"default".to_string()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn round_trips_through_json() {
		let mut config = Config::new("style/tailwind.css", "src/components/ui");
		config.registries.insert(
			"@acme".to_string(),
			"https://acme.dev/r/{name}.json".to_string(),
		);

		let json = serde_json::to_string(&config).unwrap();
		let parsed: Config = serde_json::from_str(&json).unwrap();

		assert_eq!(parsed, config);
		assert!(json.starts_with("{\"$schema\":"));
	}

	#[test]
	fn fills_defaults_for_a_minimal_file() {
		let parsed: Config =
			serde_json::from_str(r#"{"tailwind":{"css":"input.css"},"paths":{"ui":"src/ui"}}"#)
				.unwrap();

		assert_eq!(parsed.style, "default");
		assert!(parsed.registries.is_empty());
	}
}
