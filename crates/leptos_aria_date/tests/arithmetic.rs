//! Date arithmetic tests over the proleptic Gregorian calendar.

use leptos_aria_date::DateDuration;
use leptos_aria_date::DateValue;

#[test]
fn months_clamp_to_their_length() {
	assert_eq!(
		DateValue::new(2023, 1, 31).add(DateDuration::months(1)),
		DateValue::new(2023, 2, 28)
	);
	assert_eq!(
		DateValue::new(2024, 1, 31).add(DateDuration::months(1)),
		DateValue::new(2024, 2, 29)
	);
}

#[test]
fn days_cross_month_ends() {
	let january = DateValue::new(2023, 1, 29);

	assert_eq!(
		january.add(DateDuration::days(3)),
		DateValue::new(2023, 2, 1)
	);
	assert_eq!(
		january.add(DateDuration::days(-29)),
		DateValue::new(2022, 12, 31)
	);
}

#[test]
fn years_respect_february_29() {
	assert_eq!(
		DateValue::new(2024, 2, 29).add(DateDuration::default().with_years(1)),
		DateValue::new(2025, 2, 28)
	);
}

#[test]
fn weekdays_match_known_dates() {
	assert_eq!(DateValue::new(2024, 1, 1).weekday(), 1, "a Monday");
	assert_eq!(DateValue::new(2026, 10, 18).weekday(), 0, "a Sunday");
}
