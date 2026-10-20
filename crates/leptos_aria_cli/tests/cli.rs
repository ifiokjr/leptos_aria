//! End-to-end: build the repository's registry, then drive `init`, `add`,
//! `diff`, `list`, and `view` against a scratch Cargo project, the way a user
//! would after `cargo install leptos_aria_cli`.

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;

/// A scratch workspace holding a built registry and a fresh Cargo project,
/// removed when dropped.
struct Scratch {
	dir: PathBuf,
	registry: PathBuf,
	project: PathBuf,
}

impl Scratch {
	/// Build the registry and create an empty binary crate. With `init`, also
	/// run `leptos-aria init --skip-deps` in it.
	fn new(name: &str, init: bool) -> Self {
		let dir =
			std::env::temp_dir().join(format!("leptos-aria-cli-{}-{name}", std::process::id()));
		let _ = fs::remove_dir_all(&dir);
		fs::create_dir_all(dir.join("app/src")).unwrap();

		let scratch = Self {
			registry: dir.join("r"),
			project: dir.join("app"),
			dir,
		};
		let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../registry/registry.json");

		ok(&scratch.run_in(
			&scratch.dir,
			&[
				"build",
				manifest.to_str().unwrap(),
				"--output",
				scratch.registry.to_str().unwrap(),
			],
		));
		fs::write(
			scratch.project.join("Cargo.toml"),
			"[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\nleptos \
			 = \"0.8\"\n",
		)
		.unwrap();
		fs::write(scratch.project.join("src/main.rs"), "fn main() {}\n").unwrap();

		if init {
			ok(&scratch.run(&["init", "--skip-deps"]));
		}

		scratch
	}

	fn run(&self, args: &[&str]) -> Output {
		self.run_in(&self.project, args)
	}

	fn run_in(&self, cwd: &Path, args: &[&str]) -> Output {
		Command::new(env!("CARGO_BIN_EXE_leptos-aria"))
			.args(args)
			.current_dir(cwd)
			.env(
				"LEPTOS_ARIA_REGISTRY_URL",
				self.registry.join("{name}.json").to_string_lossy().as_ref(),
			)
			.output()
			.unwrap()
	}

	fn file(&self, path: &str) -> String {
		read(self.project.join(path))
	}
}

impl Drop for Scratch {
	fn drop(&mut self) {
		let _ = fs::remove_dir_all(&self.dir);
	}
}

fn ok(output: &Output) -> String {
	let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
	assert!(
		output.status.success(),
		"command failed\nstdout:\n{stdout}\nstderr:\n{}",
		String::from_utf8_lossy(&output.stderr)
	);
	stdout
}

fn read(path: impl AsRef<Path>) -> String {
	fs::read_to_string(path.as_ref())
		.unwrap_or_else(|error| panic!("{}: {error}", path.as_ref().display()))
}

#[test]
fn build_inlines_contents_into_items_but_not_the_index() {
	let scratch = Scratch::new("build", false);

	assert!(read(scratch.registry.join("badge.json")).contains("\"content\""));
	assert!(!read(scratch.registry.join("registry.json")).contains("\"content\""));
}

#[test]
fn init_writes_config_theme_helper_and_modules() {
	let scratch = Scratch::new("init", false);

	let init = ok(&scratch.run(&["init", "--skip-deps"]));
	assert!(init.contains("cargo add tw_merge leptos_aria"), "{init}");
	assert!(
		scratch
			.file("leptos-aria.json")
			.contains("\"ui\": \"src/components/ui\"")
	);

	let css = scratch.file("style/tailwind.css");
	assert!(css.starts_with("@import \"tailwindcss\";"), "{css}");
	assert!(css.contains("/* leptos-aria: default */"));
	assert!(css.contains("@custom-variant dark (&:is(.dark *));"));
	assert!(css.contains("\t--background: oklch(1 0 0);"));
	assert!(css.contains("\t--color-primary: var(--primary);"));

	assert!(
		scratch
			.file("src/components/ui/utils.rs")
			.contains("pub fn cn(")
	);
	assert_eq!(scratch.file("src/components/ui/mod.rs"), "pub mod utils;\n");
	assert_eq!(scratch.file("src/components/mod.rs"), "pub mod ui;\n");
	assert_eq!(
		scratch.file("src/main.rs"),
		"fn main() {}\nmod components;\n"
	);

	assert!(
		!scratch.run(&["init"]).status.success(),
		"a second init needs --force"
	);
}

#[test]
fn add_is_idempotent_and_keeps_local_edits() {
	let scratch = Scratch::new("add", true);

	let add = ok(&scratch.run(&["add", "badge", "card", "--skip-deps"]));
	assert!(
		add.contains("created      src/components/ui/badge.rs"),
		"{add}"
	);
	assert!(
		add.contains("unchanged    src/components/ui/utils.rs"),
		"{add}"
	);
	assert_eq!(
		scratch.file("src/components/ui/mod.rs"),
		"pub mod utils;\npub mod badge;\npub mod card;\n"
	);

	let repeat = ok(&scratch.run(&["add", "badge", "--skip-deps"]));
	assert!(
		repeat.contains("unchanged    src/components/ui/badge.rs"),
		"{repeat}"
	);
	assert_eq!(
		scratch
			.file("style/tailwind.css")
			.matches("leptos-aria: default */")
			.count(),
		1
	);

	let badge = scratch.project.join("src/components/ui/badge.rs");
	fs::write(&badge, read(&badge).replace("rounded-md", "rounded-full")).unwrap();

	let kept = ok(&scratch.run(&["add", "badge", "--skip-deps"]));
	assert!(
		kept.contains("skipped      src/components/ui/badge.rs"),
		"{kept}"
	);

	let changed = ok(&scratch.run(&["diff"]));
	assert!(changed.lines().any(|line| line == "badge"), "{changed}");

	let diff = ok(&scratch.run(&["diff", "badge"]));
	assert!(
		diff.lines()
			.any(|line| line.starts_with('-') && line.contains("rounded-full")),
		"{diff}"
	);

	let restored = ok(&scratch.run(&["add", "badge", "--overwrite", "--skip-deps"]));
	assert!(
		restored.contains("overwrote    src/components/ui/badge.rs"),
		"{restored}"
	);
	assert!(ok(&scratch.run(&["diff"])).contains("Every installed component matches"));
}

#[test]
fn list_view_and_dry_run_read_without_writing() {
	let scratch = Scratch::new("read", true);

	let list = ok(&scratch.run(&["list"]));
	assert!(
		list.lines().any(|line| line.starts_with("badge ")),
		"{list}"
	);

	let view = ok(&scratch.run(&["view", "kbd"]));
	assert!(
		view.contains("\"name\": \"kbd\"") && view.contains("pub fn Kbd("),
		"{view}"
	);

	let dry = ok(&scratch.run(&["add", "skeleton", "--dry-run"]));
	assert!(dry.contains("Dry run"), "{dry}");
	assert!(
		!scratch
			.project
			.join("src/components/ui/skeleton.rs")
			.exists()
	);
}

#[test]
fn reports_unknown_items_and_namespaces() {
	let scratch = Scratch::new("errors", true);

	let missing = scratch.run(&["add", "nope"]);
	assert!(!missing.status.success());
	assert!(String::from_utf8_lossy(&missing.stderr).contains("nope.json"));

	let namespace = scratch.run(&["add", "@acme/card"]);
	assert!(String::from_utf8_lossy(&namespace.stderr).contains("unknown registry `@acme`"));
}
