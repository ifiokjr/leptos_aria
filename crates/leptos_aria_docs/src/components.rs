//! The component registry's pages: an index of every styled component and
//! one page per component with install commands, usage, and source.
//!
//! Everything comes from `registry/registry.json` and the component files
//! themselves. A component's usage section is its `//!` module doc, so the
//! example a user reads on the site is the one copied into their project.
//! That example is also what the page previews live: `leptos_aria_preview`
//! compiles each one with [`usage_example`] and the page frames it in an
//! iframe next to its code.

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

/// Split module-doc markdown around its usage example, the first `rust`
/// fence: the prose before it, the example code, and the prose after it.
/// `None` when there is no complete `rust` fence.
pub fn split_usage_example(markdown: &str) -> Option<(&str, &str, &str)> {
	let (fence_start, code_start) = find_line(markdown, 0, "```rust")?;
	let (code_end, after) = find_line(markdown, code_start, "```")?;

	Some((
		markdown.get(..fence_start)?,
		markdown.get(code_start..code_end)?,
		markdown.get(after..)?,
	))
}

/// The byte range, newline included, of the first line at or after `from`
/// that reads `text` once trimmed.
fn find_line(markdown: &str, from: usize, text: &str) -> Option<(usize, usize)> {
	let mut start = from;

	for line in markdown.get(from..)?.split_inclusive('\n') {
		let end = start + line.len();

		if line.trim() == text {
			return Some((start, end));
		}

		start = end;
	}

	None
}

/// The code of a component's usage example, read from its source file's
/// module docs.
pub fn usage_example(source: &str) -> Option<String> {
	let docs = module_docs(source);

	split_usage_example(&docs).map(|(_, code, _)| code.to_string())
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
		 class=\"install\"><code tabindex=\"0\"><span class=\"prompt\" aria-hidden=\"true\">$</span> \
		 {command}</code><button class=\"copy\" type=\"button\" \
		 data-copy=\"{command}\">Copy</button></div>\n",
		title = escape_html(component.title),
		description = escape_html(component.description),
	);

	html.push_str(&manual_steps(component));
	html.push_str("<h2 id=\"usage\">Usage</h2>\n");

	let docs = module_docs(component.source);

	if let Some((before, code, after)) = split_usage_example(&docs) {
		html.push_str(&markdown_to_html(before));
		html.push_str(&preview_html(component, code));
		html.push_str(&markdown_to_html(after));
	} else {
		html.push_str(&markdown_to_html(&docs));
	}

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

/// The usage example as shadcn-style tabs: a live preview framed in an
/// iframe, and the code it runs.
///
/// The iframe isolates the registry's own Tailwind theme and preflight from
/// the site's styles, and `loading="lazy"` keeps the preview's wasm off the
/// network until the frame nears the viewport. Without scripting the tabs
/// cannot switch and the preview cannot run, so a `<noscript>` style shows
/// only the code.
fn preview_html(component: &ComponentDoc<'_>, code: &str) -> String {
	format!(
		"<figure class=\"preview\" data-preview>\n<div class=\"preview-tabs\" role=\"tablist\" \
		 aria-label=\"{title} example\">\n<button type=\"button\" role=\"tab\" \
		 id=\"preview-tab-{name}\" aria-controls=\"preview-panel-{name}\" \
		 aria-selected=\"true\">Preview</button>\n<button type=\"button\" role=\"tab\" \
		 id=\"code-tab-{name}\" aria-controls=\"code-panel-{name}\" aria-selected=\"false\" \
		 tabindex=\"-1\">Code</button>\n</div>\n<div class=\"preview-stage\" role=\"tabpanel\" \
		 id=\"preview-panel-{name}\" aria-labelledby=\"preview-tab-{name}\">\n<iframe \
		 src=\"../preview/index.html?example={name}\" title=\"{title} example\" \
		 loading=\"lazy\"></iframe>\n</div>\n<div class=\"preview-code\" role=\"tabpanel\" \
		 id=\"code-panel-{name}\" aria-labelledby=\"code-tab-{name}\" hidden>\n<pre><code \
		 class=\"language-rust\">{code}</code></pre>\n</div>\n<noscript><style>.preview-tabs, \
		 .preview-stage {{ display: none; }} .preview-code[hidden] {{ display: block; \
		 }}</style></noscript>\n</figure>\n",
		name = escape_html(component.name),
		title = escape_html(component.title),
		code = escape_html(code),
	)
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
	fn splits_the_first_rust_fence_out_of_the_docs() {
		let docs = "Intro.\n\n```rust\nview! {}\n```\n\nNotes.\n\n```rust\nmore\n```\n";

		assert_eq!(
			split_usage_example(docs),
			Some((
				"Intro.\n\n",
				"view! {}\n",
				"\nNotes.\n\n```rust\nmore\n```\n"
			))
		);
		assert_eq!(split_usage_example("```bash\nls\n```\n"), None);
		assert_eq!(split_usage_example("```rust\nunclosed\n"), None);
		assert_eq!(
			usage_example(SOURCE).as_deref(),
			Some("view! { <Badge>\"New\"</Badge> }\n")
		);
	}

	#[test]
	fn frames_the_usage_example_as_a_lazy_preview() {
		let html = page_html(&badge(&[], &[]));
		let preview = html.find("<figure class=\"preview\"").expect("a preview");
		let code = html.find("id=\"code-panel-badge\"").expect("a code panel");

		assert!(html.contains(
			"<iframe src=\"../preview/index.html?example=badge\" title=\"Badge example\" \
			 loading=\"lazy\">"
		));
		assert!(html.contains("aria-controls=\"code-panel-badge\" aria-selected=\"false\""));
		assert!(code > preview);
		assert!(html[code..].contains("view! { &lt;Badge&gt;"));
		assert_eq!(html.matches("<figure").count(), 1);
	}

	#[test]
	fn links_every_component_from_the_index() {
		let html = index_html(&[badge(&[], &[])]);

		assert!(html.contains("<a href=\"components/badge.html\"><strong>Badge</strong>"));
	}
}
