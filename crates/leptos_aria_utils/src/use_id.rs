use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;

static NEXT_ID: AtomicU32 = AtomicU32::new(1);

/// Generate a unique id for labeling and description wiring.
///
/// Ids are prefixed so they cannot collide with user-supplied ids in
/// `aria-labelledby` references.
pub fn use_id() -> String {
	let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);

	format!("leptos-aria-{id}")
}
