//! Static site assembly for the `leptos_aria` website.
//!
//! The website is one layout template, a hand-written landing page, and
//! markdown pages rendered from the repository docs, so the site and the repo
//! never drift. These functions are pure; the `site` binary does the file IO.

/// The deployed site's root URL, used for canonical links and the sitemap.
pub const SITE_URL: &str = "https://ifiokjr.github.io/leptos_aria/";

/// What the layout template needs to know about one page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PageMeta<'a> {
	/// Short page identifier, exposed as `data-page` on `<body>` so the
	/// navigation can mark the current page with CSS alone.
	pub slug: &'a str,
	/// Output path relative to the site root; empty for the landing page.
	pub path: &'a str,
	/// Document title.
	pub title: &'a str,
	/// Meta description, also used for social cards.
	pub description: &'a str,
}

/// Fill the layout's `{{slug}}`, `{{url}}`, `{{title}}`, `{{description}}`,
/// and `{{content}}` placeholders.
///
/// Metadata is HTML-escaped; `content` is inserted verbatim and last, so
/// placeholder-like text inside a page is never substituted.
pub fn render_layout(layout: &str, page: &PageMeta<'_>, content: &str) -> String {
	let url = format!("{SITE_URL}{}", page.path);

	layout
		.replace("{{slug}}", &escape_html(page.slug))
		.replace("{{url}}", &escape_html(&url))
		.replace("{{title}}", &escape_html(page.title))
		.replace("{{description}}", &escape_html(page.description))
		.replace("{{content}}", content)
}

/// Build `llms-full.txt`: the `llms.txt` index followed by every document in
/// full, each under a `## <label>` heading.
pub fn compose_llms_full(index: &str, documents: &[(&str, &str)]) -> String {
	let mut output = String::from(index.trim_end());
	output.push_str("\n\n---\n\n# Full documentation\n");

	for (label, markdown) in documents {
		output.push_str("\n## ");
		output.push_str(label);
		output.push_str("\n\n");
		output.push_str(skip_leading_html(markdown).trim_end());
		output.push('\n');
	}

	output
}

/// A `sitemap.xml` listing each page path under [`SITE_URL`].
pub fn sitemap(paths: &[&str]) -> String {
	let mut output = String::from(
		"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset \
		 xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
	);

	for path in paths {
		output.push_str("\t<url><loc>");
		output.push_str(&escape_html(&format!("{SITE_URL}{path}")));
		output.push_str("</loc></url>\n");
	}

	output.push_str("</urlset>\n");
	output
}

/// Drop the raw HTML banner (logo, badges) a readme opens with, so agents
/// reading the plain-text docs start at the first heading.
fn skip_leading_html(markdown: &str) -> &str {
	let mut offset = 0;

	for line in markdown.split_inclusive('\n') {
		let trimmed = line.trim();

		if !(trimmed.is_empty() || trimmed.starts_with('<')) {
			break;
		}

		offset += line.len();
	}

	markdown.get(offset..).unwrap_or(markdown)
}

/// Escape the five characters that are unsafe in HTML text and attributes.
fn escape_html(text: &str) -> String {
	let mut escaped = String::with_capacity(text.len());

	for char in text.chars() {
		match char {
			'&' => escaped.push_str("&amp;"),
			'<' => escaped.push_str("&lt;"),
			'>' => escaped.push_str("&gt;"),
			'"' => escaped.push_str("&quot;"),
			'\'' => escaped.push_str("&#39;"),
			_ => escaped.push(char),
		}
	}

	escaped
}

#[cfg(test)]
mod tests {
	use super::*;

	const PAGE: PageMeta<'static> = PageMeta {
		slug: "tailwind",
		path: "tailwind.html",
		title: "Tailwind <v4> guide",
		description: "Style \"state\" & more",
	};

	#[test]
	fn fills_and_escapes_layout_placeholders() {
		let layout = "<title>{{title}}</title><meta content=\"{{description}}\"><link \
		              href=\"{{url}}\"><body data-page=\"{{slug}}\">{{content}}</body>";
		let html = render_layout(layout, &PAGE, "<p>{{title}}</p>");

		assert_eq!(
			html,
			"<title>Tailwind &lt;v4&gt; guide</title><meta content=\"Style &quot;state&quot; \
			 &amp; more\"><link href=\"https://ifiokjr.github.io/leptos_aria/tailwind.html\"><body \
			 data-page=\"tailwind\"><p>{{title}}</p></body>"
		);
	}

	#[test]
	fn composes_llms_full_without_readme_banners() {
		let readme =
			"<p align=\"center\">\n\t<img src=\"logo.svg\">\n</p>\n\n# leptos_aria\n\nBody.\n";
		let full = compose_llms_full("# Index\n\nSummary.\n\n", &[("readme", readme)]);

		assert_eq!(
			full,
			"# Index\n\nSummary.\n\n---\n\n# Full documentation\n\n## readme\n\n# \
			 leptos_aria\n\nBody.\n"
		);
	}

	#[test]
	fn keeps_documents_without_a_banner_intact() {
		assert_eq!(skip_leading_html("# Title\n<br>\n"), "# Title\n<br>\n");
		assert_eq!(skip_leading_html("<p>only html</p>"), "");
	}

	#[test]
	fn lists_every_page_in_the_sitemap() {
		let xml = sitemap(&["", "roadmap.html"]);

		assert!(xml.contains("<loc>https://ifiokjr.github.io/leptos_aria/</loc>"));
		assert!(xml.contains("<loc>https://ifiokjr.github.io/leptos_aria/roadmap.html</loc>"));
		assert!(xml.ends_with("</urlset>\n"));
	}
}
