//! Markdown rendering with embedded component fences — placeholder until the
//! full renderer lands.

/// Parse a markdown document into segments of markdown and component fences.
pub fn parse_docs(markdown: &str) -> Vec<String> {
	markdown.lines().map(str::to_string).collect()
}
