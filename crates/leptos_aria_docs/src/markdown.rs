//! Markdown with embedded component fences for the `leptos_aria` docs.
//!
//! Docs pages are plain markdown except for fenced code blocks whose info
//! string starts with `component:`. The fence body carries `key: value`
//! props and the page renders a live `leptos_aria` component in place of the
//! block:
//!
//! `````markdown
//! ```component:Button
//! # Lines starting with `#` are comments.
//! label: Delete everything
//! variant: destructive
//! ```
//! `````
//!
//! The parser stays markup-agnostic: it yields markdown segments and
//! component blocks, and the docs site decides how each one renders. The
//! markdown segments convert to HTML with pulldown-cmark, with strikethrough
//! and tables enabled, and every heading gets a slug `id` so sections can be
//! linked to directly.

use std::collections::HashMap;

use pulldown_cmark::CowStr;
use pulldown_cmark::Event;
use pulldown_cmark::Options;
use pulldown_cmark::Parser;
use pulldown_cmark::Tag;
use pulldown_cmark::TagEnd;
use pulldown_cmark::html;

/// Info-string marker that turns a code fence into a component block.
pub const COMPONENT_PREFIX: &str = "component:";

/// One contiguous run of a docs page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DocsSegment {
	/// Markdown source, to be rendered as HTML.
	Markdown(String),
	/// A `component:` fence, to be rendered as a live component.
	Component(ComponentBlock),
}

/// A parsed `component:` fence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentBlock {
	/// Component name taken from the fence info string, for example `Button`.
	pub name: String,
	/// Ordered `key: value` props taken from the fence body.
	pub props: Vec<ComponentProp>,
}

/// A single `key: value` line from a component fence body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentProp {
	/// Prop name.
	pub key: String,
	/// Prop value with surrounding whitespace removed.
	pub value: String,
}

impl ComponentBlock {
	/// Look up a prop value by key.
	pub fn prop(&self, key: &str) -> Option<&str> {
		self.props
			.iter()
			.find(|prop| prop.key == key)
			.map(|prop| prop.value.as_str())
	}
}

/// Split a docs page into markdown and component-fence segments.
///
/// Empty runs (leading, trailing, or between fences) produce no segment. A
/// component fence left open runs to the end of the input.
pub fn parse_docs(markdown: &str) -> Vec<DocsSegment> {
	let mut segments: Vec<DocsSegment> = Vec::new();
	let mut text = String::new();
	let mut fence: Option<ComponentBlock> = None;

	for line in markdown.lines() {
		if let Some(block) = fence.as_mut() {
			if is_closing_fence(line) {
				segments.push(DocsSegment::Component(fence.take().expect("block is set")));
			} else if let Some(prop) = parse_prop(line) {
				block.props.push(prop);
			}

			continue;
		}

		if let Some(name) = component_fence_name(line) {
			push_markdown(&mut segments, &mut text);
			fence = Some(ComponentBlock {
				name: name.to_string(),
				props: Vec::new(),
			});
		} else {
			text.push_str(line);
			text.push('\n');
		}
	}

	if let Some(block) = fence {
		segments.push(DocsSegment::Component(block));
	}

	push_markdown(&mut segments, &mut text);
	segments
}

/// Convert a markdown segment to HTML with strikethrough and tables enabled.
///
/// Headings without an explicit id get a slug of their text, deduplicated
/// with a numeric suffix (`usage`, `usage-1`), so every section is linkable.
pub fn markdown_to_html(markdown: &str) -> String {
	let mut options = Options::empty();
	options.insert(Options::ENABLE_STRIKETHROUGH);
	options.insert(Options::ENABLE_TABLES);

	let mut events: Vec<Event<'_>> = Parser::new_ext(markdown, options).collect();
	assign_heading_ids(&mut events);

	let mut output = String::with_capacity(markdown.len() * 3 / 2);
	html::push_html(&mut output, events.into_iter());
	output
}

/// Give every heading that lacks an id a unique slug of its text.
fn assign_heading_ids(events: &mut [Event<'_>]) {
	let mut seen: HashMap<String, usize> = HashMap::new();

	for index in 0..events.len() {
		let Some(Event::Start(Tag::Heading { id: None, .. })) = events.get(index) else {
			continue;
		};

		let text = heading_text(events.get(index + 1..).unwrap_or_default());
		let base = slugify(&text);
		let count = seen.entry(base.clone()).or_insert(0);
		let slug = if *count == 0 {
			base
		} else {
			format!("{base}-{count}")
		};
		*count += 1;

		if let Some(Event::Start(Tag::Heading { id, .. })) = events.get_mut(index) {
			*id = Some(CowStr::from(slug));
		}
	}
}

/// The plain text of a heading, read from the events that follow its start.
fn heading_text(events: &[Event<'_>]) -> String {
	events
		.iter()
		.take_while(|event| !matches!(event, Event::End(TagEnd::Heading(_))))
		.filter_map(|event| {
			match event {
				Event::Text(text) | Event::Code(text) => Some(text.as_ref()),
				_ => None,
			}
		})
		.collect()
}

/// Lowercase alphanumerics joined by single dashes: `Or name the variants`
/// becomes `or-name-the-variants`.
fn slugify(text: &str) -> String {
	let mut slug = String::with_capacity(text.len());

	for char in text.chars().flat_map(char::to_lowercase) {
		if char.is_alphanumeric() {
			slug.push(char);
		} else if !slug.is_empty() && !slug.ends_with('-') {
			slug.push('-');
		}
	}

	if slug.ends_with('-') {
		slug.pop();
	}

	if slug.is_empty() {
		slug.push_str("section");
	}

	slug
}

/// Finish the current markdown run, if it has content.
fn push_markdown(segments: &mut Vec<DocsSegment>, text: &mut String) {
	if text.trim().is_empty() {
		text.clear();

		return;
	}

	segments.push(DocsSegment::Markdown(std::mem::take(text)));
}

/// Read the info string of an opening backtick fence, if the line is one.
fn fence_info(line: &str) -> Option<&str> {
	let trimmed = line.trim_start();
	let backticks = trimmed.chars().take_while(|&char| char == '`').count();

	if backticks < 3 {
		return None;
	}

	Some(trimmed[backticks..].trim())
}

/// Detect a fence line that closes a component block.
fn is_closing_fence(line: &str) -> bool {
	line.trim().chars().all(|char| char == '`') && line.trim().len() >= 3
}

/// The component name when the line opens a `component:` fence.
fn component_fence_name(line: &str) -> Option<&str> {
	let info = fence_info(line)?;
	info.strip_prefix(COMPONENT_PREFIX)
		.map(str::trim)
		.filter(|name| !name.is_empty())
}

/// Parse one `key: value` fence-body line, skipping blanks and comments.
fn parse_prop(line: &str) -> Option<ComponentProp> {
	let trimmed = line.trim();

	if trimmed.is_empty() || trimmed.starts_with('#') {
		return None;
	}

	let (key, value) = trimmed.split_once(':')?;

	Some(ComponentProp {
		key: key.trim().to_string(),
		value: value.trim().to_string(),
	})
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn splits_markdown_around_a_component_fence() {
		let page =
			"# Heading\n\nText before.\n\n```component:Button\nlabel: Go\n```\n\nText after.\n";
		let segments = parse_docs(page);

		assert_eq!(segments.len(), 3);
		assert_eq!(
			segments[0],
			DocsSegment::Markdown("# Heading\n\nText before.\n\n".to_string())
		);
		assert_eq!(
			segments[1],
			DocsSegment::Component(ComponentBlock {
				name: "Button".to_string(),
				props: vec![ComponentProp {
					key: "label".to_string(),
					value: "Go".to_string(),
				}],
			})
		);
		assert_eq!(
			segments[2],
			DocsSegment::Markdown("\nText after.\n".to_string())
		);
	}

	#[test]
	fn parses_props_with_comments_blanks_and_extra_colons() {
		let page =
			"```component:Dialog\n# a comment\n\nsize : large\n\nlabel: Warning: high voltage\n```";
		let segments = parse_docs(page);

		let DocsSegment::Component(block) = &segments[0] else {
			panic!("expected a component segment");
		};

		assert_eq!(block.name, "Dialog");
		assert_eq!(block.prop("size"), Some("large"));
		assert_eq!(block.prop("label"), Some("Warning: high voltage"));
		assert_eq!(block.prop("missing"), None);
	}

	#[test]
	fn keeps_plain_code_fences_as_markdown() {
		let page = "Example:\n\n```rust\nlet x = 1;\n```\n";
		let segments = parse_docs(page);

		assert_eq!(segments.len(), 1);
		assert!(matches!(segments[0], DocsSegment::Markdown(_)));
	}

	#[test]
	fn runs_an_unclosed_fence_to_the_end_of_input() {
		let page = "Intro.\n\n```component:Switch\nchecked: true";
		let segments = parse_docs(page);

		assert_eq!(segments.len(), 2);
		let DocsSegment::Component(block) = &segments[1] else {
			panic!("expected a component segment");
		};

		assert_eq!(block.prop("checked"), Some("true"));
	}

	#[test]
	fn renders_markdown_to_html() {
		let html = markdown_to_html("# Title\n\nSome ~~struck~~ **bold** text.\n");

		assert!(html.contains("<h1 id=\"title\">Title</h1>"));
		assert!(html.contains("<del>struck</del>"));
		assert!(html.contains("<strong>bold</strong>"));
	}

	#[test]
	fn renders_tables() {
		let html = markdown_to_html("| prop | type |\n| --- | --- |\n| label | string |\n");

		assert!(html.contains("<table>"));
		assert!(html.contains("<td>label</td>"));
	}

	#[test]
	fn slugs_headings_with_code_and_punctuation() {
		let html = markdown_to_html("## The `data-state` attribute — v4!\n");

		assert!(html.contains("<h2 id=\"the-data-state-attribute-v4\">"));
	}

	#[test]
	fn deduplicates_repeated_heading_slugs() {
		let html = markdown_to_html("## Usage\n\n## Usage\n\n## Usage\n");

		assert!(html.contains("id=\"usage\""));
		assert!(html.contains("id=\"usage-1\""));
		assert!(html.contains("id=\"usage-2\""));
	}

	#[test]
	fn slugify_trims_and_falls_back_for_symbol_only_text() {
		assert_eq!(slugify("—"), "section");
		assert_eq!(slugify("  Hello,  World  "), "hello-world");
	}
}
