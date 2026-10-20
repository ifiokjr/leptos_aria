//! The component registry's pages: an index of every styled component and
//! one page per component with install commands, usage, and source.
//!
//! Everything comes from `registry/registry.json` and the component files
//! themselves. A component's usage section is its `//!` module doc, so the
//! example a user reads on the site is the one copied into their project.

use std::fmt::Write;

use crate::markdown_to_html;
use crate::site::escape_html;

/// One `registry:ui` item, as the pages need it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentDoc<'a> {
	/// The `leptos-aria add` name, also the page slug.
	pub name: &'a str,
	/// Display name.
	pub title: &'a str,
	/// One-sentence summary.
	pub description: &'a str,
	/// Crates the component needs.
	pub dependencies: &'a [String],
	/// Other registry items installed with it.
	pub registry_dependencies: &'a [String],
	/// The component's file name, such as `button.rs`.
	pub file_name: &'a str,
	/// The component's full source.
	pub source: &'a str,
}

/// The markdown in a file's leading `//!` doc comment, with doctest-only
/// fence attributes (`rust ignore`) reduced to `rust` for display.
pub fn module_docs(source: &str) -> String {
	let mut markdown = String::new();

	for line in source.lines() {
		let Some(doc) = line.strip_prefix("//!") else {
			break;
		};
		let doc = doc.strip_prefix(' ').unwrap_or(doc);
		let doc = if doc.trim_start().starts_with("```rust") {
			"```rust"
		} else {
			doc
		};

		markdown.push_str(doc);
		markdown.push('\n');
	}

	markdown
}

/// The body of `components.html`: how to start, then a card per component.
pub fn index_html(components: &[ComponentDoc<'_>]) -> String {
	let mut html = String::from(
		"<article class=\"prose wide\">\n<h1>Components</h1>\n<p class=\"lede\">Styled, accessible \
		 components you copy into your project and own. Behaviour comes from the headless \
		 <code>leptos_aria</code> crates; the Tailwind classes are yours to change.</p>\n<pre \
		 class=\"terminal\"><code><span class=\"tok-c\"># once per project</span>\n<span \
		 class=\"tok-p\">$</span> cargo install leptos_aria_cli\n<span class=\"tok-p\">$</span> \
		 leptos-aria init\n\n<span class=\"tok-c\"># then per component</span>\n<span \
		 class=\"tok-p\">$</span> leptos-aria add button dialog</code></pre>\n<p>See the <a \
		 href=\"cli.html\">CLI guide</a> for configuration, theming, and hosting your own \
		 registry.</p>\n</article>\n<ul class=\"cards component-grid\">\n",
	);

	for component in components {
		// Writing into a String cannot fail.
		let _ = writeln!(
			html,
			"<li><a href=\"components/{name}.html\"><strong>{title}</strong><span>{description}</span></a></li>",
			name = escape_html(component.name),
			title = escape_html(component.title),
			description = escape_html(component.description),
		);
	}

	html.push_str("</ul>\n");
	html
}

/// The body of `components/<name>.html`.
pub fn page_html(component: &ComponentDoc<'_>) -> String {
	let name = escape_html(component.name);
	let command = format!("leptos-aria add {name}");
	let mut html = format!(
		"<article class=\"prose\">\n<p class=\"eyebrow\"><a href=\"../components.html\">Components</a></p>\n<h1>{title}</h1>\n<p \
		 class=\"lede\">{description}</p>\n<h2 id=\"installation\">Installation</h2>\n<div \
		 class=\"install\"><code><span class=\"prompt\" aria-hidden=\"true\">$</span> \
		 {command}</code><button class=\"copy\" type=\"button\" \
		 data-copy=\"{command}\">Copy</button></div>\n",
		title = escape_html(component.title),
		description = escape_html(component.description),
	);

	html.push_str(&manual_steps(component));
	html.push_str("<h2 id=\"usage\">Usage</h2>\n");
	html.push_str(&markdown_to_html(&module_docs(component.source)));
	// Writing into a String cannot fail.
	let _ = write!(
		html,
		"<h2 id=\"source\">Source</h2>\n<p><code>src/components/ui/{file}</code></p>\n<pre><code \
		 class=\"language-rust\">{source}</code></pre>\n</article>\n",
		file = escape_html(component.file_name),
		source = escape_html(component.source),
	);

	html
}

/// The "without the CLI" steps: copy the file, declare the module, add the
/// crates and sibling items.
fn manual_steps(component: &ComponentDoc<'_>) -> String {
	let module = component
		.file_name
		.strip_suffix(".rs")
		.unwrap_or(component.file_name);
	let mut steps = format!(
		"<details>\n<summary>Install by hand</summary>\n<ol>\n<li>Copy the source below into \
		 <code>src/components/ui/{file}</code>.</li>\n<li>Add <code>pub mod {module};</code> to \
		 <code>src/components/ui/mod.rs</code>.</li>\n",
		file = escape_html(component.file_name),
		module = escape_html(module),
	);

	if !component.dependencies.is_empty() {
		// Writing into a String cannot fail.
		let _ = writeln!(
			steps,
			"<li>Run <code>cargo add {}</code>.</li>",
			escape_html(&component.dependencies.join(" "))
		);
	}

	let siblings: Vec<String> = component
		.registry_dependencies
		.iter()
		.map(|dependency| {
			if dependency.contains('/') || dependency.contains(':') {
				format!("<code>{}</code>", escape_html(dependency))
			} else {
				format!(
					"<a href=\"{name}.html\"><code>{name}</code></a>",
					name = escape_html(dependency)
				)
			}
		})
		.collect();

	if !siblings.is_empty() {
		// Writing into a String cannot fail.
		let _ = writeln!(
			steps,
			"<li>Install what it builds on: {}.</li>",
			siblings.join(", ")
		);
	}

	steps.push_str("</ol>\n</details>\n");
	steps
}

#[cfg(test)]
mod tests {
	use super::*;

	const SOURCE: &str = "//! A small status label.\n//!\n//! ```rust ignore\n//! view! { \
	                      <Badge>\"New\"</Badge> }\n//! ```\n\nuse leptos::prelude::*;\n";

	fn badge<'a>(dependencies: &'a [String], registry: &'a [String]) -> ComponentDoc<'a> {
		ComponentDoc {
			name: "badge",
			title: "Badge",
			description: "A small status label",
			dependencies,
			registry_dependencies: registry,
			file_name: "badge.rs",
			source: SOURCE,
		}
	}

	#[test]
	fn reads_the_leading_module_docs_only() {
		assert_eq!(
			module_docs(SOURCE),
			"A small status label.\n\n```rust\nview! { <Badge>\"New\"</Badge> }\n```\n"
		);
		assert_eq!(module_docs("use x;\n//! not leading\n"), "");
	}

	#[test]
	fn renders_install_usage_and_escaped_source() {
		let dependencies = vec!["leptos_aria".to_string()];
		let registry = vec!["utils".to_string()];
		let html = page_html(&badge(&dependencies, &registry));

		assert!(html.contains("data-copy=\"leptos-aria add badge\""));
		assert!(html.contains("<code>pub mod badge;</code>"));
		assert!(html.contains("<code>cargo add leptos_aria</code>"));
		assert!(html.contains("<a href=\"utils.html\"><code>utils</code></a>"));
		assert!(html.contains("<code class=\"language-rust\">//! A small status label."));
		assert!(html.contains("&lt;Badge&gt;"));
	}

	#[test]
	fn links_every_component_from_the_index() {
		let html = index_html(&[badge(&[], &[])]);

		assert!(html.contains("<a href=\"components/badge.html\"><strong>Badge</strong>"));
	}
}
