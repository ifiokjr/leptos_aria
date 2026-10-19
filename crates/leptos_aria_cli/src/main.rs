use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use clap::Parser;
use clap::Subcommand;
use serde::Deserialize;
use serde::Serialize;

/// The shadcn-style registry for leptos_aria.
#[derive(Parser)]
#[command(
	name = "leptos-aria",
	version,
	about = "Add leptos_aria components to your project"
)]
struct Cli {
	#[command(subcommand)]
	command: Command,
}

#[derive(Subcommand)]
enum Command {
	/// Write a leptos-aria.json config into the current project.
	Init,
	/// List every component in the registry.
	List,
	/// Copy registry components into your project.
	Add {
		/// The components to add, by name.
		components: Vec<String>,
	},
}

/// One registry entry.
#[derive(Deserialize, Serialize)]
struct RegistryItem {
	name: String,
	title: String,
	description: String,
	dependencies: Vec<String>,
	files: Vec<RegistryFile>,
}

/// One file of a registry entry.
#[derive(Deserialize, Serialize)]
struct RegistryFile {
	path: String,
	target: String,
}

/// The user's project configuration.
#[derive(Serialize, Deserialize)]
struct Config {
	/// Where copied components land, `src/aria` by default.
	#[serde(rename = "componentsDir")]
	components_dir: String,
	/// The workspace crate that receives the components.
	#[serde(rename = "targetCrate")]
	target_crate: String,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			components_dir: "src/aria".to_string(),
			target_crate: String::new(),
		}
	}
}

/// The embedded registry manifest.
const REGISTRY: &str = include_str!("../../../registry/registry.json");

/// The embedded component sources, keyed by registry name.
const SOURCES: &[(&str, &str)] = &[
	(
		"button",
		include_str!("../../../registry/components/button/src/aria/button.rs"),
	),
	(
		"dialog",
		include_str!("../../../registry/components/dialog/src/aria/dialog.rs"),
	),
	(
		"switch",
		include_str!("../../../registry/components/switch/src/aria/switch.rs"),
	),
];

fn main() {
	let cli = Cli::parse();

	match cli.command {
		Command::Init => init(),
		Command::List => list(),
		Command::Add { components } => add(&components),
	}
}

fn init() {
	let config = Config::default();
	let json = serde_json::to_string_pretty(&config).expect("config serializes");

	fs::write("leptos-aria.json", format!("{json}\n")).expect("write leptos-aria.json");

	println!("Created leptos-aria.json — components copy into src/aria.");
}

fn list() {
	for name in registry_items().keys() {
		println!("{name}");
	}
}

fn add(components: &[String]) {
	let items = registry_items();

	if components.is_empty() {
		eprintln!("No components named. Try: leptos-aria add button");

		return;
	}

	let config = read_config();
	let root = project_root();
	let mut written = Vec::new();

	for name in components {
		let Some(item) = items.get(name) else {
			eprintln!("Unknown component: {name}");

			continue;
		};

		for file in &item.files {
			let Some(source) = source_of(name) else {
				eprintln!("No embedded source for {name}");

				continue;
			};

			let target = target_path(&root, &config, file);
			let parent = target.parent().unwrap_or_else(|| Path::new("."));

			fs::create_dir_all(parent).expect("create component directory");
			fs::write(&target, source).expect("write component file");

			written.push(target);
		}
	}

	if written.is_empty() {
		eprintln!("Nothing was added.");

		return;
	}

	for path in &written {
		println!("Added {}", path.display());
	}

	println!(
		"Dependencies to add: {}",
		registry_items()
			.values()
			.filter(|item| components.contains(&item.name))
			.flat_map(|item| item.dependencies.iter().cloned())
			.collect::<Vec<_>>()
			.join(", ")
	);
}

type RegistryMap = BTreeMap<String, RegistryItem>;

/// The manifest envelope: metadata plus the item list.
#[derive(Deserialize)]
struct Registry {
	items: Vec<RegistryItem>,
}

fn registry_items() -> RegistryMap {
	let registry: Registry = serde_json::from_str(REGISTRY).expect("embedded registry parses");

	registry
		.items
		.into_iter()
		.map(|item| (item.name.clone(), item))
		.collect()
}

fn source_of(name: &str) -> Option<&'static str> {
	SOURCES
		.iter()
		.find(|(key, _)| *key == name)
		.map(|(_, source)| *source)
}

fn read_config() -> Config {
	fs::read_to_string("leptos-aria.json")
		.ok()
		.and_then(|text| serde_json::from_str(&text).ok())
		.unwrap_or_default()
}

fn project_root() -> PathBuf {
	PathBuf::from(".")
}

fn target_path(root: &Path, config: &Config, file: &RegistryFile) -> PathBuf {
	let dir = if config.target_crate.is_empty() {
		config.components_dir.clone()
	} else {
		format!("{}/{}", config.target_crate, config.components_dir)
	};

	let file_name = file
		.target
		.rsplit('/')
		.next()
		.unwrap_or(&file.target)
		.to_string();

	root.join(dir).join(file_name)
}
