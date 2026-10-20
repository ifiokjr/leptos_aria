#![allow(clippy::needless_pass_by_value)]

use leptos::prelude::*;
use leptos_aria_calendar::Calendar;
use leptos_aria_date::DateValue;
use leptos_aria_utils::merge_class;

/// A date range between a start and an end date.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DateRange {
	/// The first day of the range, inclusive.
	pub start: DateValue,

	/// The last day of the range, inclusive.
	pub end: DateValue,
}

impl DateRange {
	/// A range of one day.
	pub fn single(day: DateValue) -> Self {
		Self {
			start: day,
			end: day,
		}
	}

	/// A range from a start to an end; the earlier date becomes the start.
	pub fn new(first: DateValue, second: DateValue) -> Self {
		if first <= second {
			Self {
				start: first,
				end: second,
			}
		} else {
			Self {
				start: second,
				end: first,
			}
		}
	}

	/// Whether a date falls inside the range, inclusively.
	pub fn contains(&self, day: DateValue) -> bool {
		self.start <= day && day <= self.end
	}

	/// The number of days in the range, counting both ends.
	pub fn day_count(&self) -> u32 {
		let days = days_between(self.start, self.end);

		days + 1
	}
}

/// The whole days between two dates.
fn days_between(from: DateValue, to: DateValue) -> u32 {
	if from == to {
		return 0;
	}

	let (earlier, later) = if from < to { (from, to) } else { (to, from) };
	let mut current = earlier;
	let mut days = 0_u32;

	while current < later {
		current = current.add(leptos_aria_date::DateDuration::days(1));
		days += 1;
	}

	days
}

/// The state of a range selection in progress.
#[derive(Clone)]
pub struct DateRangeState {
	range: RwSignal<DateRange>,
}

impl DateRangeState {
	/// The selected range.
	///
	/// Reactive: reading inside an effect or view tracks the range.
	pub fn range(&self) -> DateRange {
		self.range.get()
	}

	/// The selected range, without tracking.
	pub fn range_untracked(&self) -> DateRange {
		self.range.get_untracked()
	}

	/// Replace the whole range.
	pub fn set_range(&self, range: DateRange) {
		self.range.set(range);
	}
}

/// A date range field shows the start and end of a range as text.
#[component]
pub fn DateRangeField(
	/// The current range (controlled).
	#[prop(into)]
	value: Signal<DateRange>,
	/// The content to display as the label.
	#[prop(optional)]
	label: Option<String>,
	/// Classes added after the built-in `leptos-aria-date-range-field` class.
	#[prop(optional, into)]
	class: Signal<String>,
) -> impl IntoView {
	view! {
		<span class=merge_class("leptos-aria-date-range-field", class) aria-label=label.unwrap_or_default()>
			{move || format_date(value.get().start)}
			" – "
			{move || format_date(value.get().end)}
		</span>
	}
}

/// The `YYYY-MM-DD` form of a date.
fn format_date(date: DateValue) -> String {
	format!("{}-{:02}-{:02}", date.year, date.month, date.day)
}

/// A date range picker combines a trigger with a range calendar.
///
/// The first click sets the start anchor, the second click completes the
/// range, and a further click starts the next range.
#[component]
pub fn DateRangePicker(
	/// The initially selected range (uncontrolled).
	#[prop(optional)]
	default_value: Option<DateRange>,
	/// The content to display as the label.
	#[prop(optional)]
	label: Option<String>,
	/// Classes added after the built-in `leptos-aria-date-range-picker` class.
	#[prop(optional, into)]
	class: Signal<String>,
) -> impl IntoView {
	let is_open = RwSignal::new(false);
	let range = RwSignal::new(
		default_value.unwrap_or_else(|| DateRange::single(DateValue::new(2026, 1, 1))),
	);

	let state = DateRangeState { range };
	let trigger_state = state.clone();
	let calendar_state = state;

	view! {
		<div class=merge_class("leptos-aria-date-range-picker", class)>
			<button
				type="button"
				class="leptos-aria-date-range-picker__trigger"
				aria-haspopup="dialog"
				aria-expanded=move || is_open.get().to_string()
				on:click=move |_| is_open.update(|open| *open = !*open)
			>
				{move || {
					let range = trigger_state.range_untracked();

					format!("{} – {}", format_date(range.start), format_date(range.end))
				}}
			</button>
			<div
				class="leptos-aria-date-range-picker__calendar"
				hidden=move || !is_open.get()
				role="dialog"
				aria-label=label.unwrap_or_else(|| "Select date range".to_string())
			>
				<RangeCalendar range=calendar_state />
			</div>
		</div>
	}
}

/// A calendar that selects a range across two clicks.
#[component]
pub fn RangeCalendar(
	range: DateRangeState,
	/// Classes added after the built-in `leptos-aria-calendar` class of the
	/// inner calendar.
	#[prop(optional, into)]
	class: Signal<String>,
) -> impl IntoView {
	view! {
		<Calendar class=class default_value=range.range_untracked().start />
		<span class="leptos-aria-range-calendar__count">
			{move || format!("{} days", range.range().day_count()) }
		</span>
	}
}
