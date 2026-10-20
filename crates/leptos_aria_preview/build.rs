//! Compile the usage example of every `registry:ui` item into a preview.
//!
//! Each component's example is the first `rust` fence in its `//!` module
//! docs, the snippet its website page shows and `leptos-aria add` copies
//! into a project. Generating the previews from those snippets keeps the
//! preview and the code beside it identical, and compiles every snippet in
//! CI so none of them can drift from the component's real API.

use std::env;
use std::fmt::Write;
use std::fs;
use std::path::Path;

use leptos_aria_docs::components::usage_example;
use serde::Deserialize;

/// The parts of `registry/registry.json` the previews read.
#[derive(Deserialize)]
struct Manifest {
	items: Vec<Item>,
}

#[derive(Deserialize)]
struct Item {
	name: String,
	#[serde(rename = "type")]
	kind: String,
	#[serde(default)]
	files: Vec<File>,
}

#[derive(Deserialize)]
struct File {
	path: String,
}

/// The module path the snippets import from in a user's project.
const PROJECT_MODULE: &str = "use crate::components::ui::";

/// The same modules, as the registry crate exposes them.
const REGISTRY_MODULE: &str = "use leptos_aria_registry::ui::";

fn main() {
	let registry = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../registry");
	let manifest = registry.join("registry.json");
	let components = registry.join("src/ui");

	println!("cargo::rerun-if-changed={}", manifest.display());
	println!("cargo::rerun-if-changed={}", components.display());

	let manifest: Manifest = serde_json::from_str(&read(&manifest)).expect("registry.json parses");
	let mut functions = String::new();
	let mut table = String::from(
		"/// Every registry component's usage example, by registry item name.\npub const \
		 EXAMPLES: &[(&str, Render)] = &[\n",
	);

	for item in manifest
		.items
		.iter()
		.filter(|item| item.kind == "registry:ui")
	{
		let name = &item.name;
		let path = &item
			.files
			.first()
			.unwrap_or_else(|| panic!("registry item `{name}` has no file"))
			.path;
		let code = usage_example(&read(&registry.join(path)))
			.unwrap_or_else(|| panic!("`{path}` has no ```rust usage example in its module docs"));
		let function = name.replace('-', "_");

		// Writing into a String cannot fail.
		let _ = write!(
			functions,
			"/// The usage example in `registry/{path}`.\npub fn {function}() -> AnyView {{\n\tlet \
			 view = {{\n{body}\t}};\n\n\tview.into_any()\n}}\n\n",
			body = indent(&code.replace(PROJECT_MODULE, REGISTRY_MODULE)),
		);
		let _ = writeln!(table, "\t(\"{name}\", {function}),");
	}

	table.push_str("];\n");

	let out = Path::new(&env::var("OUT_DIR").expect("cargo sets OUT_DIR")).join("examples.rs");
	fs::write(out, functions + &table).expect("the generated examples are written");
}

fn read(path: &Path) -> String {
	fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

/// Indent every non-blank line by two tabs, to sit inside the generated block.
fn indent(code: &str) -> String {
	code.lines()
		.map(|line| {
			if line.trim().is_empty() {
				"\n".to_string()
			} else {
				format!("\t\t{line}\n")
			}
		})
		.collect()
}
