//! Build the `leptos_aria` website into a static directory.
//!
//! ```bash
//! cargo run -p leptos_aria_docs --bin site            # writes target/site
//! cargo run -p leptos_aria_docs --bin site -- out/dir # writes out/dir
//! ```
//!
//! `website/public` is copied verbatim, `website/pages/index.html` and the
//! repository markdown docs are rendered into `website/layout.html`, and
//! `llms-full.txt`, `registry.json`, and `sitemap.xml` are generated from the
//! same sources so nothing on the site is a hand-maintained copy.

use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use leptos_aria_docs::markdown_to_html;
use leptos_aria_docs::site::PageMeta;
use leptos_aria_docs::site::compose_llms_full;
use leptos_aria_docs::site::render_layout;
use leptos_aria_docs::site::sitemap;

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
		slug: "roadmap",
		source: "PORTING.md",
		title: "Porting roadmap · leptos_aria",
		description: "Every React Aria hook, utility, and component and the leptos_aria crate that \
		              ports it.",
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

	let llms = fs::read_to_string(website.join("public/llms.txt"))?;
	let readme = fs::read_to_string(root.join("readme.md"))?;
	let tailwind = fs::read_to_string(root.join("docs/tailwind.md"))?;
	let roadmap = fs::read_to_string(root.join("PORTING.md"))?;
	let documents = [
		("readme", readme.as_str()),
		("Tailwind v4 guide", tailwind.as_str()),
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
