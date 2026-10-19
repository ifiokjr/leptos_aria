//! Date range tests over concrete calendar dates.

use leptos_aria_date::DateValue;
use leptos_aria_date_picker::DateRange;

#[test]
fn ranges_normalize_order_and_count_days() {
	let range = DateRange::new(DateValue::new(2026, 10, 20), DateValue::new(2026, 10, 18));

	assert_eq!(range.start, DateValue::new(2026, 10, 18));
	assert_eq!(range.end, DateValue::new(2026, 10, 20));
	assert_eq!(range.day_count(), 3, "both ends count");
	assert!(range.contains(DateValue::new(2026, 10, 19)));
	assert!(!range.contains(DateValue::new(2026, 10, 21)));
}

#[test]
fn ranges_span_month_ends() {
	let range = DateRange::new(DateValue::new(2026, 10, 30), DateValue::new(2026, 11, 2));

	assert_eq!(range.day_count(), 4);
}
