//! Copy registry items into a project: write their files, declare the Rust
//! modules so they compile, extend the Tailwind CSS file, and `cargo add`
//! their crates.

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;

use crate::config::Config;
use crate::css;
use crate::registry::RegistryFile;
use crate::registry::RegistryItem;

/// How `add` treats the project.
#[derive(Clone, Copy, Debug, Default)]
pub struct InstallOptions {
	/// Replace files that differ from the registry.
	pub overwrite: bool,
	/// Report what would change without writing anything.
	pub dry_run: bool,
	/// Leave `Cargo.toml` alone and only report the crates to add.
	pub skip_deps: bool,
}

/// One change `add` made or would make.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Change {
	/// A file was written.
	Created(PathBuf),
	/// An existing file was replaced (`--overwrite`).
	Overwritten(PathBuf),
	/// The file already matches the registry.
	Unchanged(PathBuf),
	/// The file differs and was kept; pass `--overwrite` to replace it.
	Skipped(PathBuf),
	/// A `mod` line was added to a module file.
	Module { file: PathBuf, line: String },
	/// The item's CSS block was written into the Tailwind file.
	Css { file: PathBuf, item: String },
	/// Crates added with `cargo add`, or to add when `--skip-deps` is set.
	Dependencies {
		crates: Vec<String>,
		installed: bool,
	},
	/// A message from the item's author.
	Docs { item: String, message: String },
}

/// Install items, which must already be ordered dependencies first.
pub fn install(
	root: &Path,
	config: &Config,
	items: &[RegistryItem],
	options: InstallOptions,
) -> Result<Vec<Change>> {
	let mut install = Install {
		root,
		config,
		ui_dir: root.join(&config.paths.ui),
		options,
		changes: Vec::new(),
		modules: Vec::new(),
		crates: Vec::new(),
		docs: Vec::new(),
	};

	for item in items {
		install.files(item)?;
		install.css(item)?;
		install.remember(item);
	}

	install.finish()
}

/// The running state of one `add`: what changed so far, and what is left to
/// do once every file is in place.
struct Install<'a> {
	root: &'a Path,
	config: &'a Config,
	ui_dir: PathBuf,
	options: InstallOptions,
	changes: Vec<Change>,
	/// Component modules to declare in the component directory.
	modules: Vec<String>,
	/// Crates to `cargo add`, in first-seen order.
	crates: Vec<String>,
	/// Author messages, reported after everything else.
	docs: Vec<Change>,
}

impl Install<'_> {
	/// Write the item's files.
	fn files(&mut self, item: &RegistryItem) -> Result<()> {
		for file in &item.files {
			let Some(content) = &file.content else {
				bail!(
					"`{}` has no inlined content for {}. Registries are served from the output of \
					 `leptos-aria build`.",
					item.name,
					file.path
				);
			};
			let target = target_path(self.root, &self.ui_dir, file);

			self.changes
				.push(write_file(&target, content, self.options)?);

			if let Some(module) = component_module(&target, &self.ui_dir)
				&& !self.modules.contains(&module)
			{
				self.modules.push(module);
			}
		}

		Ok(())
	}

	/// Write the item's CSS block into the Tailwind file.
	fn css(&mut self, item: &RegistryItem) -> Result<()> {
		let Some(block) = css::render_item_css(item) else {
			return Ok(());
		};
		let file = self.root.join(&self.config.tailwind.css);

		if !self.options.dry_run {
			let existing =
				fs::read_to_string(&file).unwrap_or_else(|_| format!("{}\n", css::TAILWIND_IMPORT));

			write(&file, &css::upsert_block(&existing, &item.name, &block))?;
		}

		self.changes.push(Change::Css {
			file,
			item: item.name.clone(),
		});

		Ok(())
	}

	/// Note the item's crates and message for the final steps.
	fn remember(&mut self, item: &RegistryItem) {
		for dependency in &item.dependencies {
			if !self.crates.contains(dependency) {
				self.crates.push(dependency.clone());
			}
		}

		if let Some(message) = &item.docs {
			self.docs.push(Change::Docs {
				item: item.name.clone(),
				message: message.clone(),
			});
		}
	}

	/// Declare modules, add crates, and append author messages.
	fn finish(mut self) -> Result<Vec<Change>> {
		if !self.modules.is_empty() {
			self.changes.extend(declare_modules(
				self.root,
				&self.config.paths.ui,
				&self.modules,
				self.options.dry_run,
			)?);
		}

		let missing = missing_dependencies(self.root, &self.crates)?;

		if !missing.is_empty() {
			let installed = !(self.options.skip_deps || self.options.dry_run);

			if installed {
				cargo_add(self.root, &missing)?;
			}

			self.changes.push(Change::Dependencies {
				crates: missing,
				installed,
			});
		}

		self.changes.extend(self.docs);

		Ok(self.changes)
	}
}

/// The module name of a `.rs` file placed directly in the component
/// directory; `None` for `mod.rs` and anything nested or non-Rust.
fn component_module(target: &Path, ui_dir: &Path) -> Option<String> {
	let is_rust = target
		.extension()
		.is_some_and(|extension| extension == "rs");

	if target.parent() != Some(ui_dir) || !is_rust {
		return None;
	}

	target
		.file_stem()
		.and_then(|stem| stem.to_str())
		.filter(|stem| *stem != "mod")
		.map(str::to_string)
}

/// Where a registry file lands: `target` with `@ui/` and `~/` expanded, or
/// the component directory when there is no target.
pub fn target_path(root: &Path, ui_dir: &Path, file: &RegistryFile) -> PathBuf {
	let Some(target) = file.target.as_deref() else {
		let name = Path::new(&file.path)
			.file_name()
			.map_or_else(|| PathBuf::from(&file.path), PathBuf::from);
		return ui_dir.join(name);
	};

	if let Some(rest) = target.strip_prefix("@ui/") {
		ui_dir.join(rest)
	} else if let Some(rest) = target.strip_prefix("~/") {
		root.join(rest)
	} else {
		root.join(target)
	}
}

/// Write one file unless it already matches, or differs and `--overwrite` was
/// not passed: local edits are never lost by accident.
fn write_file(target: &Path, content: &str, options: InstallOptions) -> Result<Change> {
	let existing = fs::read_to_string(target).ok();
	let change = match existing {
		Some(existing) if existing == content => {
			return Ok(Change::Unchanged(target.to_path_buf()));
		}
		Some(_) if !options.overwrite => return Ok(Change::Skipped(target.to_path_buf())),
		Some(_) => Change::Overwritten(target.to_path_buf()),
		None => Change::Created(target.to_path_buf()),
	};

	if !options.dry_run {
		write(target, content)?;
	}

	Ok(change)
}

fn write(path: &Path, content: &str) -> Result<()> {
	if let Some(parent) = path.parent() {
		fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
	}

	fs::write(path, content).with_context(|| format!("writing {}", path.display()))
}

/// Declare each component module in the component directory's module file,
/// and every directory between `src/` and it in its parent, so copied files
/// compile without touching anything by hand.
fn declare_modules(
	root: &Path,
	ui: &str,
	modules: &[String],
	dry_run: bool,
) -> Result<Vec<Change>> {
	let mut changes = Vec::new();
	let segments: Vec<&str> = ui
		.split('/')
		.filter(|segment| !segment.is_empty())
		.collect();
	let ui_dir = root.join(ui);

	for module in modules {
		changes.extend(ensure_mod_line(
			&module_file(&ui_dir),
			module,
			"pub mod",
			dry_run,
		)?);
	}

	// Wire `src/components/ui` up through `src/components` to the crate root.
	let Some(src_index) = segments.iter().position(|segment| *segment == "src") else {
		return Ok(changes);
	};
	let src = segments
		.get(..=src_index)
		.map_or_else(|| root.join("src"), |parts| root.join(parts.join("/")));
	let nested = segments.get(src_index + 1..).unwrap_or_default();

	for (depth, segment) in nested.iter().enumerate() {
		let (parent_file, keyword) = if depth == 0 {
			crate_root(&src)
		} else {
			let parent_dir = nested
				.get(..depth)
				.map_or_else(|| src.clone(), |parts| src.join(parts.join("/")));
			(module_file(&parent_dir), "pub mod")
		};

		changes.extend(ensure_mod_line(&parent_file, segment, keyword, dry_run)?);
	}

	Ok(changes)
}

/// `src/lib.rs` (`pub mod`) when it exists, otherwise `src/main.rs` (`mod`).
fn crate_root(src: &Path) -> (PathBuf, &'static str) {
	let lib = src.join("lib.rs");

	if lib.exists() {
		(lib, "pub mod")
	} else {
		(src.join("main.rs"), "mod")
	}
}

/// The file that declares a directory's children: `dir.rs` if it exists,
/// otherwise `dir/mod.rs`.
fn module_file(dir: &Path) -> PathBuf {
	let sibling = dir.with_extension("rs");

	if sibling.exists() {
		sibling
	} else {
		dir.join("mod.rs")
	}
}

fn ensure_mod_line(
	file: &Path,
	module: &str,
	keyword: &str,
	dry_run: bool,
) -> Result<Option<Change>> {
	let existing = fs::read_to_string(file).unwrap_or_default();

	if declares_module(&existing, module) {
		return Ok(None);
	}

	let line = format!("{keyword} {module};");

	if !dry_run {
		let mut updated = existing;

		if !updated.is_empty() && !updated.ends_with('\n') {
			updated.push('\n');
		}

		updated.push_str(&line);
		updated.push('\n');
		write(file, &updated)?;
	}

	Ok(Some(Change::Module {
		file: file.to_path_buf(),
		line,
	}))
}

/// Whether any line already declares `mod <module>;`, at any visibility.
fn declares_module(source: &str, module: &str) -> bool {
	let declaration = format!("mod {module};");

	source.lines().any(|line| {
		let line = line.trim();
		let line = line
			.strip_prefix("pub(crate) ")
			.or_else(|| line.strip_prefix("pub "))
			.unwrap_or(line);
		line == declaration
	})
}

/// The crates not yet in `[dependencies]`, keeping their `@version` suffix.
fn missing_dependencies(root: &Path, crates: &[String]) -> Result<Vec<String>> {
	if crates.is_empty() {
		return Ok(Vec::new());
	}

	let manifest_path = root.join("Cargo.toml");
	let text = fs::read_to_string(&manifest_path)
		.with_context(|| format!("reading {}", manifest_path.display()))?;
	let manifest: toml::Table = text
		.parse()
		.with_context(|| format!("parsing {}", manifest_path.display()))?;
	let declared = manifest.get("dependencies").and_then(toml::Value::as_table);

	Ok(crates
		.iter()
		.filter(|spec| {
			let name = spec.split('@').next().unwrap_or(spec);
			!declared.is_some_and(|table| table.contains_key(name))
		})
		.cloned()
		.collect())
}

fn cargo_add(root: &Path, crates: &[String]) -> Result<()> {
	let status = Command::new("cargo")
		.arg("add")
		.args(crates)
		.current_dir(root)
		.status()
		.context("running `cargo add`")?;

	if !status.success() {
		bail!("`cargo add {}` failed", crates.join(" "));
	}

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	fn file(path: &str, target: Option<&str>) -> RegistryFile {
		RegistryFile {
			path: path.to_string(),
			kind: "registry:ui".to_string(),
			target: target.map(str::to_string),
			content: None,
		}
	}

	#[test]
	fn resolves_targets() {
		let root = Path::new("/app");
		let ui = Path::new("/app/src/components/ui");

		assert_eq!(
			target_path(root, ui, &file("src/ui/button.rs", None)),
			Path::new("/app/src/components/ui/button.rs")
		);
		assert_eq!(
			target_path(root, ui, &file("x.rs", Some("@ui/forms/field.rs"))),
			Path::new("/app/src/components/ui/forms/field.rs")
		);
		assert_eq!(
			target_path(root, ui, &file("x", Some("~/.env.example"))),
			Path::new("/app/.env.example")
		);
		assert_eq!(
			target_path(root, ui, &file("x", Some("style/extra.css"))),
			Path::new("/app/style/extra.css")
		);
	}

	#[test]
	fn recognises_module_declarations_at_any_visibility() {
		assert!(declares_module("mod a;\npub mod ui;\n", "ui"));
		assert!(declares_module("  pub(crate) mod ui;", "ui"));
		assert!(!declares_module("pub mod uix;\n// mod ui;", "ui"));
	}
}
