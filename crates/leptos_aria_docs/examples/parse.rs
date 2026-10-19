//! Parse a docs page with an embedded component fence and print the segments.

use leptos_aria_docs::DocsSegment;
use leptos_aria_docs::markdown_to_html;
use leptos_aria_docs::parse_docs;

fn main() {
	let page = "\
# Button

Buttons trigger actions. Style them from the `data-*` attributes.

```component:Button
# The label prop renders inside the button.
label: Delete everything
variant: destructive
```

Press it to see focus, hover, and press states.
";

	for segment in parse_docs(page) {
		match segment {
			DocsSegment::Markdown(source) => {
				println!(
					"--- markdown segment rendered as:\n{}",
					markdown_to_html(&source)
				);
			}

			DocsSegment::Component(block) => {
				println!(
					"--- live component `{}` with label {:?}",
					block.name,
					block.prop("label").unwrap_or_default()
				);
			}
		}
	}
}
