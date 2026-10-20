//! Build the `leptos_aria` website into a static directory.
//!
//! ```bash
//! cargo run -p leptos_aria_docs --bin site            # writes target/site
//! cargo run -p leptos_aria_docs --bin site -- out/dir # writes out/dir
//! ```
//!
//! `website/public` is copied verbatim, `website/pages/index.html` and the
//! repository markdown docs are rendered into `website/layout.html`, the
//! component pages are generated from `registry/registry.json` and the
//! component sources, and `llms-full.txt` and `sitemap.xml` are composed from
//! the same files, so nothing on the site is a hand-maintained copy. The
//! installable `r/*.json` files are built separately by `leptos-aria build`.

use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use leptos_aria_docs::components::ComponentDoc;
use leptos_aria_docs::components::index_html;
use leptos_aria_docs::components::page_html;
use leptos_aria_docs::markdown_to_html;
use leptos_aria_docs::site::PageMeta;
use leptos_aria_docs::site::compose_llms_full;
use leptos_aria_docs::site::render_layout;
use leptos_aria_docs::site::sitemap;
use serde::Deserialize;

/// A repository markdown document published as a site page.
struct MarkdownPage {
	slug: &'static str,
	source: &'static str,
	title: &'static str,
	description: &'static str,
}

const MARKDOWN_PAGES: [MarkdownPage; 2] = [
	MarkdownPage {
		slug: "tailwind",
		source: "docs/tailwind.md",
		title: "Tailwind v4 guide · leptos_aria",
		description: "Style headless leptos_aria components with Tailwind v4 data-attribute variants \
		              and @custom-variant recipes, plus the full attribute contract.",
	},
	MarkdownPage {
		slug: "cli",
		source: "docs/cli.md",
		title: "CLI · leptos_aria",
		description: "Copy styled, accessible components into your Leptos project with the \
		              shadcn-style leptos-aria CLI, and host your own registry.",
	},
];

const INDEX_TITLE: &str = "leptos_aria · accessible by default, styled by you";
const INDEX_DESCRIPTION: &str = "Headless, accessible Leptos components ported from React Aria. \
                                 Zero CSS ships; every state is a data-* attribute Tailwind v4 \
                                 styles directly.";

fn main() -> io::Result<()> {
	let root = Path::new(env!("CARGO_MANIFEST_DIR"))
		.join("../..")
		.canonicalize()?;
	let out = env::args_os()
		.nth(1)
		.map_or_else(|| root.join("target/site"), PathBuf::from);
	let website = root.join("website");
	let layout = fs::read_to_string(website.join("layout.html"))?;

	copy_dir(&website.join("public"), &out)?;
	fs::copy(
		root.join("registry/registry.json"),
		out.join("registry.json"),
	)?;

	let index = PageMeta {
		slug: "index",
		path: "",
		title: INDEX_TITLE,
		description: INDEX_DESCRIPTION,
	};
	let landing = fs::read_to_string(website.join("pages/index.html"))?;
	fs::write(
		out.join("index.html"),
		render_layout(&layout, &index, &landing),
	)?;

	let mut paths = vec![String::new()];

	for page in &MARKDOWN_PAGES {
		let path = format!("{}.html", page.slug);
		let markdown = fs::read_to_string(root.join(page.source))?;
		let article = format!(
			"<article class=\"prose\">\n{}</article>\n",
			markdown_to_html(&markdown)
		);
		let meta = PageMeta {
			slug: page.slug,
			path: &path,
			title: page.title,
			description: page.description,
		};

		fs::write(out.join(&path), render_layout(&layout, &meta, &article))?;
		paths.push(path);
	}

	paths.extend(write_components(&root, &out, &layout)?);

	let llms = fs::read_to_string(website.join("public/llms.txt"))?;
	let readme = fs::read_to_string(root.join("readme.md"))?;
	let tailwind = fs::read_to_string(root.join("docs/tailwind.md"))?;
	let cli = fs::read_to_string(root.join("docs/cli.md"))?;
	let roadmap = fs::read_to_string(root.join("PORTING.md"))?;
	let documents = [
		("readme", readme.as_str()),
		("Tailwind v4 guide", tailwind.as_str()),
		("CLI", cli.as_str()),
		("Porting roadmap", roadmap.as_str()),
	];
	fs::write(
		out.join("llms-full.txt"),
		compose_llms_full(&llms, &documents),
	)?;

	let paths: Vec<&str> = paths.iter().map(String::as_str).collect();
	fs::write(out.join("sitemap.xml"), sitemap(&paths))?;

	println!("site written to {}", out.display());
	Ok(())
}

/// Recursively copy every file under `from` into `to`.
fn copy_dir(from: &Path, to: &Path) -> io::Result<()> {
	fs::create_dir_all(to)?;

	for entry in fs::read_dir(from)? {
		let entry = entry?;
		let target = to.join(entry.file_name());

		if entry.file_type()?.is_dir() {
			copy_dir(&entry.path(), &target)?;
		} else {
			fs::copy(entry.path(), target)?;
		}
	}

	Ok(())
}

/// The parts of `registry/registry.json` the component pages read.
#[derive(Deserialize)]
struct Manifest {
	items: Vec<ManifestItem>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManifestItem {
	name: String,
	#[serde(rename = "type")]
	kind: String,
	#[serde(default)]
	title: String,
	#[serde(default)]
	description: String,
	#[serde(default)]
	dependencies: Vec<String>,
	#[serde(default)]
	registry_dependencies: Vec<String>,
	#[serde(default)]
	files: Vec<ManifestFile>,
}

#[derive(Deserialize)]
struct ManifestFile {
	path: String,
}

/// Write `components.html` and one page per `registry:ui` item, returning
/// their paths.
fn write_components(root: &Path, out: &Path, layout: &str) -> io::Result<Vec<String>> {
	let text = fs::read_to_string(root.join("registry/registry.json"))?;
	let manifest: Manifest = serde_json::from_str(&text).map_err(io::Error::other)?;
	let mut sources = Vec::new();

	for item in manifest
		.items
		.iter()
		.filter(|item| item.kind == "registry:ui")
	{
		let Some(file) = item.files.first() else {
			continue;
		};
		let source = fs::read_to_string(root.join("registry").join(&file.path))?;
		let file_name = Path::new(&file.path)
			.file_name()
			.and_then(|name| name.to_str())
			.unwrap_or_default()
			.to_string();

		sources.push((item, file_name, source));
	}

	let docs: Vec<ComponentDoc<'_>> = sources
		.iter()
		.map(|(item, file_name, source)| {
			ComponentDoc {
				name: &item.name,
				title: &item.title,
				description: &item.description,
				dependencies: &item.dependencies,
				registry_dependencies: &item.registry_dependencies,
				file_name,
				source,
			}
		})
		.collect();

	let index = PageMeta {
		slug: "components",
		path: "components.html",
		title: "Components · leptos_aria",
		description: "Styled, accessible Leptos components you copy into your project with the \
		              leptos-aria CLI, shadcn-style.",
	};
	fs::write(
		out.join("components.html"),
		render_layout(layout, &index, &index_html(&docs)),
	)?;
	fs::create_dir_all(out.join("components"))?;

	let mut paths = vec!["components.html".to_string()];

	for doc in &docs {
		let path = format!("components/{}.html", doc.name);
		let title = format!("{} · leptos_aria components", doc.title);
		let meta = PageMeta {
			slug: "components",
			path: &path,
			title: &title,
			description: doc.description,
		};

		fs::write(
			out.join(&path),
			render_layout(layout, &meta, &page_html(doc)),
		)?;
		paths.push(path);
	}

	Ok(paths)
}
