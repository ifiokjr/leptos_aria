//! Write an item's `cssVars` and `css` into the project's Tailwind CSS file.
//!
//! Each item owns one marked block, so installing again replaces the block
//! instead of duplicating it, and everything outside the markers stays yours:
//!
//! ```css
//! /* leptos-aria: default */
//! :root { --background: oklch(1 0 0); }
//! /* leptos-aria: default end */
//! ```

use serde_json::Map;
use serde_json::Value;

use crate::registry::RegistryItem;

/// The first line of every new CSS file `init` creates.
pub const TAILWIND_IMPORT: &str = "@import \"tailwindcss\";";

/// The CSS an item contributes, or `None` when it has none.
pub fn render_item_css(item: &RegistryItem) -> Option<String> {
	let mut output = String::new();

	if let Some(css) = &item.css {
		render_rules(css, 0, &mut output, RuleFilter::AtStatements);
	}

	if let Some(vars) = &item.css_vars {
		render_vars(":root", &vars.light, &mut output);
		render_vars(".dark", &vars.dark, &mut output);
		render_vars("@theme inline", &vars.theme, &mut output);
	}

	if let Some(css) = &item.css {
		render_rules(css, 0, &mut output, RuleFilter::Blocks);
	}

	(!output.is_empty()).then_some(output)
}

/// Insert or replace the item's marked block in `existing`.
pub fn upsert_block(existing: &str, name: &str, css: &str) -> String {
	let start = format!("/* leptos-aria: {name} */");
	let end = format!("/* leptos-aria: {name} end */");
	let block = format!("{start}\n{}\n{end}", css.trim_end());

	if let (Some(from), Some(to)) = (existing.find(&start), existing.find(&end))
		&& from < to
	{
		let mut output = String::with_capacity(existing.len() + block.len());
		output.push_str(existing.get(..from).unwrap_or_default());
		output.push_str(&block);
		output.push_str(existing.get(to + end.len()..).unwrap_or_default());

		return output;
	}

	let mut output = existing.trim_end().to_string();

	if !output.is_empty() {
		output.push_str("\n\n");
	}

	output.push_str(&block);
	output.push('\n');
	output
}

/// Which top-level rules a pass renders. Statements such as
/// `@custom-variant` come first so they precede the variables that use them.
#[derive(Clone, Copy, PartialEq, Eq)]
enum RuleFilter {
	AtStatements,
	Blocks,
	All,
}

fn render_vars(selector: &str, vars: &indexmap::IndexMap<String, String>, output: &mut String) {
	if vars.is_empty() {
		return;
	}

	output.push_str(selector);
	output.push_str(" {\n");

	for (name, value) in vars {
		output.push_str("\t--");
		output.push_str(name);
		output.push_str(": ");
		output.push_str(value);
		output.push_str(";\n");
	}

	output.push_str("}\n");
}

/// Render shadcn's nested-object CSS: a string value is a declaration, an
/// empty object after an `@` key is a statement (`@apply ...;`), and any other
/// object is a block.
fn render_rules(rules: &Map<String, Value>, depth: usize, output: &mut String, filter: RuleFilter) {
	for (key, value) in rules {
		let statement = is_statement(key, value);
		let skip = match filter {
			RuleFilter::AtStatements => !statement,
			RuleFilter::Blocks => statement,
			RuleFilter::All => false,
		};

		if !skip {
			render_rule(key, value, statement, depth, output);
		}
	}
}

/// An `@` rule with no body, such as `@apply ...` or `@custom-variant ...`.
fn is_statement(key: &str, value: &Value) -> bool {
	key.starts_with('@') && value.as_object().is_some_and(Map::is_empty)
}

/// Append `text` on its own line, indented one tab per level.
fn push_line(output: &mut String, depth: usize, text: &str) {
	for _ in 0..depth {
		output.push('\t');
	}

	output.push_str(text);
	output.push('\n');
}

fn render_rule(key: &str, value: &Value, statement: bool, depth: usize, output: &mut String) {
	if statement {
		push_line(output, depth, &format!("{key};"));
		return;
	}

	match value {
		Value::Object(children) => {
			push_line(output, depth, &format!("{key} {{"));
			render_rules(children, depth + 1, output, RuleFilter::All);
			push_line(output, depth, "}");
		}
		Value::String(text) => push_line(output, depth, &format!("{key}: {text};")),
		other => push_line(output, depth, &format!("{key}: {other};")),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn item(json: &str) -> RegistryItem {
		serde_json::from_str(json).unwrap()
	}

	#[test]
	fn renders_variables_and_nested_rules() {
		let css = render_item_css(&item(
			r#"{
				"name": "default",
				"type": "registry:style",
				"cssVars": {
					"theme": {"color-background": "var(--background)"},
					"light": {"background": "oklch(1 0 0)"},
					"dark": {"background": "oklch(0.145 0 0)"}
				},
				"css": {
					"@custom-variant dark (&:is(.dark *))": {},
					"@layer base": {"body": {"@apply bg-background text-foreground": {}}}
				}
			}"#,
		))
		.unwrap();

		assert_eq!(
			css,
			"@custom-variant dark (&:is(.dark *));\n:root {\n\t--background: oklch(1 0 0);\n}\n.dark \
			 {\n\t--background: oklch(0.145 0 0);\n}\n@theme inline {\n\t--color-background: \
			 var(--background);\n}\n@layer base {\n\tbody {\n\t\t@apply bg-background \
			 text-foreground;\n\t}\n}\n"
		);
	}

	#[test]
	fn items_without_css_render_nothing() {
		assert_eq!(
			render_item_css(&item(r#"{"name":"a","type":"registry:ui"}"#)),
			None
		);
	}

	#[test]
	fn upserts_a_marked_block_idempotently() {
		let first = upsert_block("@import \"tailwindcss\";\n", "default", ":root {}\n");
		assert_eq!(
			first,
			"@import \"tailwindcss\";\n\n/* leptos-aria: default */\n:root {}\n/* leptos-aria: \
			 default end */\n"
		);

		let second = upsert_block(&first, "default", ".dark {}\n");
		assert_eq!(
			second,
			"@import \"tailwindcss\";\n\n/* leptos-aria: default */\n.dark {}\n/* leptos-aria: \
			 default end */\n"
		);
		assert_eq!(upsert_block(&second, "default", ".dark {}\n"), second);
	}

	/// The website's preview page is styled as a project after `init`: its
	/// stylesheet must carry the `default` style block exactly as `init` would
	/// write it from the registry, so the previews never drift from what users
	/// install.
	#[test]
	fn the_preview_stylesheet_carries_the_default_style() {
		let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
		let read = |path: &str| std::fs::read_to_string(root.join(path)).unwrap();
		let registry: crate::registry::Registry =
			serde_json::from_str(&read("registry/registry.json")).unwrap();
		let style = registry
			.items
			.iter()
			.find(|item| item.name == "default")
			.unwrap();
		let stylesheet = read("crates/leptos_aria_preview/style.css");

		assert_eq!(
			upsert_block(&stylesheet, "default", &render_item_css(style).unwrap()),
			stylesheet,
			"update the `default` block in crates/leptos_aria_preview/style.css"
		);
	}
}
