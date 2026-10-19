#![doc(html_logo_url = "https://ifiokjr.github.io/leptos_aria/logo.svg")]
#![doc(html_favicon_url = "https://ifiokjr.github.io/leptos_aria/favicon.ico")]

use std::collections::HashSet;
use std::sync::Arc;

use leptos::prelude::*;

/// The keys currently selected in a collection.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Selection {
	/// No keys are selected.
	#[default]
	None,
	/// Every key is selected, including ones not yet loaded.
	All,
	/// Exactly these keys are selected.
	Keys(HashSet<String>),
}

impl Selection {
	/// Whether the given key is selected.
	pub fn contains(&self, key: &str) -> bool {
		match self {
			Selection::None => false,
			Selection::All => true,
			Selection::Keys(keys) => keys.contains(key),
		}
	}

	/// Whether anything at all is selected.
	pub fn is_empty(&self) -> bool {
		match self {
			Selection::None => true,
			Selection::All => false,
			Selection::Keys(keys) => keys.is_empty(),
		}
	}
}

/// How many items a collection allows selecting.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum SelectionMode {
	/// No selection at all.
	#[default]
	None,
	/// A single item at a time.
	Single,
	/// Any number of items.
	Multiple,
}

/// A callback invoked when the selection changes.
pub type BoxedSelectionCallback = Box<dyn Fn(&Selection) + Send + Sync + 'static>;

/// The change callback storage of the selection state.
type SelectionChangeCallback = Arc<dyn Fn(&Selection) + Send + Sync>;

/// State management for collection selection.
///
/// The same shape as the upstream `SelectionManager` over
/// `useMultipleSelectionState`: single selection replaces, multiple selection
/// toggles, and disabled selection is a no-op.
#[derive(Clone)]
pub struct SelectionState {
	selection: SelectionValue,
	mode: SelectionMode,
	is_disabled: bool,
	on_change: Option<SelectionChangeCallback>,
}

#[derive(Clone, Copy)]
enum SelectionValue {
	Controlled(Signal<Selection>),
	Uncontrolled(RwSignal<Selection>),
}

/// The options accepted by [`use_selection_state`].
pub struct UseSelectionStateProps {
	/// The initial selection (uncontrolled).
	pub default_selection: Option<Selection>,

	/// The current selection (controlled).
	pub selection: Option<Signal<Selection>>,

	/// How many items can be selected at once.
	pub mode: SelectionMode,

	/// Whether selection is disabled.
	pub is_disabled: bool,

	/// Handler that is called when the selection changes.
	pub on_change: Option<BoxedSelectionCallback>,
}

impl Default for UseSelectionStateProps {
	fn default() -> Self {
		Self {
			default_selection: None,
			selection: None,
			mode: SelectionMode::None,
			is_disabled: false,
			on_change: None,
		}
	}
}

/// Provides state management for collection selection.
#[allow(clippy::needless_pass_by_value)]
pub fn use_selection_state(props: UseSelectionStateProps) -> SelectionState {
	let selection = match props.selection {
		Some(controlled) => SelectionValue::Controlled(controlled),
		None => {
			SelectionValue::Uncontrolled(RwSignal::new(props.default_selection.unwrap_or_default()))
		}
	};

	let on_change = props.on_change.map(|callback| {
		let callback: SelectionChangeCallback =
			Arc::new(move |selection: &Selection| callback(selection));

		callback
	});

	SelectionState {
		selection,
		mode: props.mode,
		is_disabled: props.is_disabled,
		on_change,
	}
}

impl SelectionState {
	/// The current selection.
	///
	/// Reactive: reading inside an effect or view tracks the selection.
	pub fn selection(&self) -> Selection {
		self.selection_signal().get()
	}

	/// The current selection, without tracking.
	pub fn selection_untracked(&self) -> Selection {
		self.selection_signal().get_untracked()
	}

	/// The signal backing the selection, for reactive bindings.
	pub fn selection_signal(&self) -> Signal<Selection> {
		match self.selection {
			SelectionValue::Controlled(signal) => signal,
			SelectionValue::Uncontrolled(signal) => Signal::from(signal),
		}
	}

	/// How many items can be selected at once.
	pub fn mode(&self) -> SelectionMode {
		self.mode
	}

	/// Whether the key is selected.
	pub fn is_selected(&self, key: &str) -> bool {
		self.selection_signal()
			.with_untracked(|selection| selection.contains(key))
	}

	/// Replace the whole selection.
	#[allow(clippy::needless_pass_by_value)]
	pub fn set_selection(&self, selection: Selection) {
		if self.is_disabled
			|| self
				.selection_signal()
				.with_untracked(|current| *current == selection)
		{
			return;
		}

		if let SelectionValue::Uncontrolled(signal) = self.selection {
			signal.set(selection.clone());
		}

		if let Some(on_change) = &self.on_change {
			on_change(&selection);
		}
	}

	/// Select or toggle a single key according to the selection mode.
	pub fn select_key(&self, key: &str, extend: bool) {
		match self.mode {
			SelectionMode::None => {}
			SelectionMode::Single => {
				self.set_selection(Selection::Keys(HashSet::from([key.to_string()])));
			}
			SelectionMode::Multiple => {
				if extend {
					self.toggle_key(key);
				} else {
					self.set_selection(Selection::Keys(HashSet::from([key.to_string()])));
				}
			}
		}
	}

	/// Toggle a key in the selection, keeping the other keys in multiple
	/// selection mode.
	pub fn toggle_key(&self, key: &str) {
		let mut keys = match self.selection_untracked() {
			Selection::Keys(keys) => keys,
			_ => HashSet::new(),
		};

		if keys.contains(key) {
			keys.remove(key);
		} else {
			keys.insert(key.to_string());
		}

		self.set_selection(Selection::Keys(keys));
	}

	/// Clear the selection.
	pub fn clear_selection(&self) {
		self.set_selection(Selection::None);
	}
}

/// The list of keys of a collection, in document order.
#[derive(Clone, Debug, Default)]
pub struct CollectionKeys(pub Vec<String>);

impl CollectionKeys {
	/// The key following the given one, wrapping past the end.
	pub fn key_after(&self, key: &str) -> Option<&str> {
		let index = self.index_of(key)?;

		self.0
			.get(index + 1)
			.map(String::as_str)
			.or(self.0.first().map(String::as_str))
	}

	/// The key before the given one, wrapping past the start.
	pub fn key_before(&self, key: &str) -> Option<&str> {
		let index = self.index_of(key)?;

		if index == 0 {
			self.0.last().map(String::as_str)
		} else {
			self.0.get(index - 1).map(String::as_str)
		}
	}

	/// The first key.
	pub fn first_key(&self) -> Option<&str> {
		self.0.first().map(String::as_str)
	}

	/// The last key.
	pub fn last_key(&self) -> Option<&str> {
		self.0.last().map(String::as_str)
	}

	/// The first key starting with the given prefix after the current key,
	/// wrapping once through the collection.
	pub fn key_for_search(&self, prefix: &str, from: &str) -> Option<&str> {
		let normalized = prefix.to_lowercase();
		let start = self.index_of(from).unwrap_or(0);
		let length = self.0.len();

		for offset in 1..=length {
			let index = (start + offset) % length;

			if let Some(key) = self.0.get(index) {
				if key.to_lowercase().starts_with(&normalized) {
					return Some(key.as_str());
				}
			}
		}

		None
	}

	fn index_of(&self, key: &str) -> Option<usize> {
		self.0.iter().position(|candidate| candidate == key)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn collection_keys_navigate_with_wraparound() {
		let keys = CollectionKeys(vec![
			"apple".to_string(),
			"banana".to_string(),
			"cherry".to_string(),
		]);

		assert_eq!(keys.key_after("apple"), Some("banana"));
		assert_eq!(
			keys.key_after("cherry"),
			Some("apple"),
			"wraps past the end"
		);
		assert_eq!(
			keys.key_before("apple"),
			Some("cherry"),
			"wraps past the start"
		);
		assert_eq!(keys.first_key(), Some("apple"));
		assert_eq!(keys.last_key(), Some("cherry"));
	}

	#[test]
	fn type_ahead_search_wraps_the_collection() {
		let keys = CollectionKeys(vec![
			"apple".to_string(),
			"banana".to_string(),
			"cherry".to_string(),
		]);

		assert_eq!(keys.key_for_search("b", "apple"), Some("banana"));
		assert_eq!(
			keys.key_for_search("a", "cherry"),
			Some("apple"),
			"wraps around"
		);
		assert_eq!(keys.key_for_search("z", "apple"), None);
	}

	#[test]
	fn single_selection_replaces_and_multiple_toggles() {
		let single = use_selection_state(UseSelectionStateProps {
			mode: SelectionMode::Single,
			..Default::default()
		});

		single.select_key("a", false);
		assert!(single.is_selected("a"));

		single.select_key("b", false);

		assert!(single.is_selected("b"));
		assert!(!single.is_selected("a"), "single selection replaces");

		let multiple = use_selection_state(UseSelectionStateProps {
			mode: SelectionMode::Multiple,
			..Default::default()
		});

		multiple.select_key("a", false);
		multiple.select_key("b", true);
		assert!(multiple.is_selected("a") && multiple.is_selected("b"));

		multiple.toggle_key("a");

		assert!(!multiple.is_selected("a"), "multiple selection toggles");
		assert!(multiple.is_selected("b"));
	}
}
