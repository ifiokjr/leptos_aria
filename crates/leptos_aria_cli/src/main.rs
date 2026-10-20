//! `leptos-aria`: copy Tailwind-styled `leptos_aria` components into a Leptos
//! project, the way the shadcn/ui CLI does for React.
//!
//! ```bash
//! leptos-aria init                 # leptos-aria.json, theme CSS, the `cn` helper
//! leptos-aria add button dialog    # copy components and their dependencies
//! leptos-aria add @acme/card       # from a namespaced registry
//! leptos-aria list                 # what the registry serves
//! leptos-aria view button          # an item's JSON, contents included
//! leptos-aria diff button          # your copy against the registry
//! leptos-aria build                # registry.json -> public/r/*.json
//! ```

mod build;
mod config;
mod css;
mod install;
mod registry;

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use clap::Parser;
use clap::Subcommand;
use similar::TextDiff;

use crate::config::CONFIG_FILE;
use crate::config::Config;
use crate::install::Change;
use crate::install::InstallOptions;

/// The command line: a project directory and one subcommand.
#[derive(Parser)]
#[command(
	name = "leptos-aria",
	version,
	about = "Add accessible, Tailwind-styled leptos_aria components to your project"
)]
struct Cli {
	/// The project directory, which contains `Cargo.toml`.
	#[arg(long, short = 'c', global = true, default_value = ".")]
	cwd: PathBuf,

	#[command(subcommand)]
	command: Command,
}

#[derive(Subcommand)]
enum Command {
	/// Write leptos-aria.json, the theme CSS variables, and the `cn` helper.
	Init {
		/// The Tailwind CSS file. Detected from common locations when omitted.
		#[arg(long)]
		css: Option<String>,
		/// The module directory that receives components.
		#[arg(long, default_value = "src/components/ui")]
		ui: String,
		/// Replace an existing leptos-aria.json.
		#[arg(long, short)]
		force: bool,
		/// Report crates to add instead of running `cargo add`.
		#[arg(long)]
		skip_deps: bool,
	},
	/// Copy components and everything they depend on into your project.
	Add {
		/// Item names, `@namespace/name`, URLs, or local `.json` paths.
		items: Vec<String>,
		/// Add every component in the registry.
		#[arg(long, short)]
		all: bool,
		/// Replace files that differ from the registry.
		#[arg(long, short)]
		overwrite: bool,
		/// Show what would change without writing anything.
		#[arg(long)]
		dry_run: bool,
		/// Report crates to add instead of running `cargo add`.
		#[arg(long)]
		skip_deps: bool,
	},
	/// List the items a registry serves.
	List {
		/// A namespace from leptos-aria.json, such as `@acme`.
		registry: Option<String>,
	},
	/// Print items as JSON, file contents included.
	View {
		/// Item addresses.
		#[arg(required = true)]
		items: Vec<String>,
	},
	/// Compare your copies with the registry.
	Diff {
		/// One item to diff in full; omit to list every changed component.
		item: Option<String>,
	},
	/// Build registry.json into static item files for hosting your own registry.
	Build {
		/// The source manifest.
		#[arg(default_value = "registry.json")]
		registry: PathBuf,
		/// Where the item files are written.
		#[arg(long, short, default_value = "public/r")]
		output: PathBuf,
		/// Fail when a bare dependency is not defined in this registry, instead of
		/// resolving it from the official one at install time.
		#[arg(long)]
		strict: bool,
	},
}

fn main() -> ExitCode {
	let cli = Cli::parse();

	match run(cli) {
		Ok(()) => ExitCode::SUCCESS,
		Err(error) => {
			eprintln!("error: {error:#}");
			ExitCode::FAILURE
		}
	}
}

fn run(cli: Cli) -> Result<()> {
	let root = cli.cwd;

	match cli.command {
		Command::Init {
			css,
			ui,
			force,
			skip_deps,
		} => init(&root, css, ui, force, skip_deps),
		Command::Add {
			items,
			all,
			overwrite,
			dry_run,
			skip_deps,
		} => {
			add(
				&root,
				items,
				all,
				InstallOptions {
					overwrite,
					dry_run,
					skip_deps,
				},
			)
		}
		Command::List { registry } => list(&root, registry.as_deref()),
		Command::View { items } => view(&root, &items),
		Command::Diff { item } => diff(&root, item.as_deref()),
		Command::Build {
			registry,
			output,
			strict,
		} => build_registry(&root.join(registry), &root.join(output), strict),
	}
}

fn init(root: &Path, css: Option<String>, ui: String, force: bool, skip_deps: bool) -> Result<()> {
	if !root.join("Cargo.toml").exists() {
		bail!(
			"no Cargo.toml in {}. Run init from your Leptos crate.",
			root.display()
		);
	}

	if root.join(CONFIG_FILE).exists() && !force {
		bail!("{CONFIG_FILE} already exists. Pass --force to replace it.");
	}

	let css = css.unwrap_or_else(|| detect_css(root));
	let config = Config::new(css, ui);
	let path = config.save(root)?;
	println!("Wrote {}", display(root, &path));

	let items = registry::fetch_tree(std::slice::from_ref(&config.style), Some(&config), root)?;
	let changes = install::install(
		root,
		&config,
		&items,
		InstallOptions {
			skip_deps,
			..InstallOptions::default()
		},
	)?;
	report(root, &changes);

	println!("\nReady. Add components with `leptos-aria add button`.");
	Ok(())
}

fn add(root: &Path, mut items: Vec<String>, all: bool, options: InstallOptions) -> Result<()> {
	let config = Config::load(root)?;

	if all {
		let index = registry::fetch_index(&registry::index_source(None, Some(&config))?)?;
		items.extend(
			index
				.items
				.into_iter()
				.filter(|item| item.kind == "registry:ui")
				.map(|item| item.name),
		);
	}

	if items.is_empty() {
		bail!("name at least one component, for example `leptos-aria add button`, or pass --all");
	}

	let resolved = registry::fetch_tree(&items, Some(&config), root)?;
	let changes = install::install(root, &config, &resolved, options)?;

	if options.dry_run {
		println!("Dry run: nothing was written.");
	}

	report(root, &changes);
	Ok(())
}

fn list(root: &Path, namespace: Option<&str>) -> Result<()> {
	let config = Config::load(root).ok();
	let index = registry::fetch_index(&registry::index_source(namespace, config.as_ref())?)?;
	let width = index
		.items
		.iter()
		.map(|item| item.name.len())
		.max()
		.unwrap_or(0);

	for item in &index.items {
		println!(
			"{:width$}  {}",
			item.name,
			item.description.as_deref().unwrap_or_default()
		);
	}

	Ok(())
}

fn view(root: &Path, items: &[String]) -> Result<()> {
	let config = Config::load(root).ok();

	for address in items {
		let item = registry::fetch_item(&registry::resolve(address, config.as_ref(), root)?)?;
		println!("{}", serde_json::to_string_pretty(&item)?);
	}

	Ok(())
}

fn diff(root: &Path, item: Option<&str>) -> Result<()> {
	let config = Config::load(root)?;
	let ui_dir = root.join(&config.paths.ui);

	let Some(address) = item else {
		let index = registry::fetch_index(&registry::index_source(None, Some(&config))?)?;
		let mut changed = 0;

		for summary in index.items.iter().filter(|item| item.kind == "registry:ui") {
			let installed = summary
				.files
				.iter()
				.any(|file| install::target_path(root, &ui_dir, file).exists());

			if !installed {
				continue;
			}

			let item =
				registry::fetch_item(&registry::resolve(&summary.name, Some(&config), root)?)?;

			if item.files.iter().any(|file| {
				let local = fs::read_to_string(install::target_path(root, &ui_dir, file))
					.unwrap_or_default();
				file.content.as_deref() != Some(local.as_str())
			}) {
				println!("{}", item.name);
				changed += 1;
			}
		}

		if changed == 0 {
			println!("Every installed component matches the registry.");
		} else {
			println!("\nRun `leptos-aria diff <name>` to see the changes.");
		}

		return Ok(());
	};

	let item = registry::fetch_item(&registry::resolve(address, Some(&config), root)?)?;

	for file in &item.files {
		let target = install::target_path(root, &ui_dir, file);
		let local = fs::read_to_string(&target)
			.with_context(|| format!("`{}` is not installed", item.name))?;
		let remote = file.content.as_deref().unwrap_or_default();
		let label = display(root, &target);

		print!(
			"{}",
			TextDiff::from_lines(local.as_str(), remote)
				.unified_diff()
				.header(&format!("{label} (yours)"), &format!("{label} (registry)"))
		);
	}

	Ok(())
}

/// The first existing Tailwind entry file, preferring one that imports
/// Tailwind; `style/tailwind.css` when none exists yet.
fn detect_css(root: &Path) -> String {
	const CANDIDATES: [&str; 6] = [
		"style/tailwind.css",
		"style/main.css",
		"styles/tailwind.css",
		"input.css",
		"tailwind.css",
		"style/input.css",
	];

	let existing: Vec<&str> = CANDIDATES
		.into_iter()
		.filter(|candidate| root.join(candidate).exists())
		.collect();

	existing
		.iter()
		.find(|candidate| {
			fs::read_to_string(root.join(candidate)).is_ok_and(|text| text.contains("tailwindcss"))
		})
		.or_else(|| existing.first())
		.map_or_else(
			|| CANDIDATES[0].to_string(),
			|candidate| (*candidate).to_string(),
		)
}

fn report(root: &Path, changes: &[Change]) {
	for change in changes {
		match change {
			Change::Created(path) => println!("  created      {}", display(root, path)),
			Change::Overwritten(path) => println!("  overwrote    {}", display(root, path)),
			Change::Unchanged(path) => println!("  unchanged    {}", display(root, path)),
			Change::Skipped(path) => {
				println!(
					"  skipped      {} (yours differs; pass --overwrite)",
					display(root, path)
				);
			}
			Change::Module { file, line } => {
				println!("  declared     `{line}` in {}", display(root, file));
			}
			Change::Css { file, item } => {
				println!("  styled       {} ({item})", display(root, file));
			}
			Change::Dependencies { crates, installed } => {
				if *installed {
					println!("  cargo add    {}", crates.join(" "));
				} else {
					println!("  add crates   cargo add {}", crates.join(" "));
				}
			}
			Change::Docs { item, message } => println!("\n{item}: {message}"),
		}
	}
}

fn display(root: &Path, path: &Path) -> String {
	path.strip_prefix(root)
		.unwrap_or(path)
		.display()
		.to_string()
}

fn build_registry(manifest: &Path, output: &Path, strict: bool) -> Result<()> {
	let built = build::build(manifest, output)?;

	if !built.external.is_empty() {
		if strict {
			bail!(
				"dependencies missing from this registry: {}",
				built.external.join(", ")
			);
		}

		println!(
			"note: resolved from the official registry at install time: {}",
			built.external.join(", ")
		);
	}

	println!(
		"Built {} files into {}.",
		built.written.len(),
		output.display()
	);
	Ok(())
}
