#![doc(html_logo_url = "https://ifiokjr.github.io/leptos_aria/logo.svg")]
#![doc(html_favicon_url = "https://ifiokjr.github.io/leptos_aria/favicon.ico")]
#![allow(clippy::needless_pass_by_value)]

use std::sync::Arc;

use leptos::prelude::*;
use leptos_aria_date::DateValue;
use leptos_aria_utils::merge_class;

/// A callback invoked when a date is selected.
pub type BoxedDateSelectCallback = Box<dyn Fn(DateValue) + Send + Sync + 'static>;

/// The month a calendar shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CalendarMonth {
	pub year: i32,
	pub month: u32,
}

impl CalendarMonth {
	/// The month containing a date.
	pub fn containing(date: DateValue) -> Self {
		Self {
			year: date.year,
			month: date.month,
		}
	}

	/// The month after this one.
	#[must_use]
	pub fn next(self) -> Self {
		if self.month == 12 {
			Self {
				year: self.year + 1,
				month: 1,
			}
		} else {
			Self {
				year: self.year,
				month: self.month + 1,
			}
		}
	}

	/// The month before this one.
	#[must_use]
	pub fn previous(self) -> Self {
		if self.month == 1 {
			Self {
				year: self.year - 1,
				month: 12,
			}
		} else {
			Self {
				year: self.year,
				month: self.month - 1,
			}
		}
	}
}

/// A calendar shows one month of dates navigable by keyboard.
///
/// Arrow keys move the focused date, page keys move by month, and home and
/// end jump to the week's edges; enter or space selects the focused date.
#[component]
pub fn Calendar(
	/// The initially selected date (uncontrolled).
	#[prop(optional)]
	default_value: Option<DateValue>,
	/// Handler that is called when a date is selected.
	#[prop(optional)]
	on_select: Option<BoxedDateSelectCallback>,
	/// Classes added after the built-in `leptos-aria-calendar` class.
	#[prop(optional, into)]
	class: Signal<String>,
) -> impl IntoView {
	let selected = RwSignal::new(default_value.unwrap_or(DateValue::new(2026, 1, 1)));
	let month = RwSignal::new(default_value.map_or(
		CalendarMonth {
			year: 2026,
			month: 1,
		},
		CalendarMonth::containing,
	));
	let focused = RwSignal::new(default_value.unwrap_or(DateValue::new(2026, 1, 1)));

	let state = Arc::new(CalendarState {
		selected,
		month,
		focused,
		on_select: Arc::new(on_select),
	});

	let label_state = Arc::clone(&state);
	let header_state = Arc::clone(&state);
	let key_state = Arc::clone(&state);
	let _grid_state = Arc::clone(&state);

	view! {
		<div
			class=merge_class("leptos-aria-calendar", class)
			role="grid"
			aria-label=move || month_label(label_state.month.get())
			on:keydown=move |event: web_sys::KeyboardEvent| {
				handle_key_down(&key_state, &event);
			}
		>
			<div class="leptos-aria-calendar__header">
				<button
					type="button"
					aria-label="Previous month"
					on:click={
						let state = Arc::clone(&state);

						move |_| state.month.update(|month| *month = month.previous())
					}
				>
					"‹"
				</button>
				<span class="leptos-aria-calendar__label">
					{move || month_label(header_state.month.get())}
				</span>
				<button
					type="button"
					aria-label="Next month"
					on:click={
						let state = Arc::clone(&state);

						move |_| state.month.update(|month| *month = month.next())
					}
				>
					"›"
				</button>
			</div>
			<MonthGrid month=state.month focused=state.focused selected=state.selected />
		</div>
	}
}

/// The shared state of a calendar.
struct CalendarState {
	selected: RwSignal<DateValue>,
	month: RwSignal<CalendarMonth>,
	focused: RwSignal<DateValue>,
	on_select: Arc<Option<BoxedDateSelectCallback>>,
}

/// The month label, e.g. "October 2026".
fn month_label(month: CalendarMonth) -> String {
	let name = match month.month {
		1 => "January",
		2 => "February",
		3 => "March",
		4 => "April",
		5 => "May",
		6 => "June",
		7 => "July",
		8 => "August",
		9 => "September",
		10 => "October",
		11 => "November",
		_ => "December",
	};

	format!("{name} {}", month.year)
}

/// The keyboard interaction of the calendar.
fn handle_key_down(state: &CalendarState, event: &web_sys::KeyboardEvent) {
	let focused = state.focused.get_untracked();

	let next = match event.key().as_str() {
		"ArrowLeft" => Some(focused.add(leptos_aria_date::DateDuration::days(-1))),
		"ArrowRight" => Some(focused.add(leptos_aria_date::DateDuration::days(1))),
		"ArrowUp" => Some(focused.add(leptos_aria_date::DateDuration::days(-7))),
		"ArrowDown" => Some(focused.add(leptos_aria_date::DateDuration::days(7))),
		"PageUp" => Some(focused.add(leptos_aria_date::DateDuration::months(-1))),
		"PageDown" => Some(focused.add(leptos_aria_date::DateDuration::months(1))),
		"Enter" | " " => {
			event.prevent_default();
			state.selected.set(focused);

			if let Some(on_select) = state.on_select.as_ref() {
				(*on_select)(focused);
			}

			return;
		}
		_ => None,
	};

	if let Some(next) = next {
		event.prevent_default();
		state.focused.set(next);
		state
			.month
			.update(|month| *month = CalendarMonth::containing(next));
	}
}

/// The grid of one month's dates.
#[component]
fn MonthGrid(
	month: RwSignal<CalendarMonth>,
	focused: RwSignal<DateValue>,
	selected: RwSignal<DateValue>,
) -> impl IntoView {
	let weeks = Signal::derive(move || month_weeks(month.get()));

	view! {
		<div class="leptos_aria-calendar__grid" role="rowgroup">
			{move || {
				weeks
					.get()
					.into_iter()
					.map(|week| {
						view! {
							<div class="leptos-aria-calendar__week" role="row">
								{week
									.into_iter()
									.map(|date| {
										let is_focused = focused.get_untracked() == date;
										let is_selected = selected.get_untracked() == date;

										view! {
											<CalendarCell date=date is_focused=is_focused is_selected=is_selected />
										}
									})
									.collect::<Vec<_>>()}
							</div>
						}
					})
					.collect::<Vec<_>>()
			}}
		</div>
	}
}

/// One date cell of the calendar.
#[component]
fn CalendarCell(date: DateValue, is_focused: bool, is_selected: bool) -> impl IntoView {
	view! {
		<span
			role="gridcell"
			aria-selected=is_selected.to_string()
			aria-current=if is_focused { "date" } else { "false" }
			class=if is_selected {
				"leptos-aria-calendar__cell leptos-aria-calendar__cell--selected"
			} else {
				"leptos-aria-calendar__cell"
			}
		>
			{date.day.to_string()}
		</span>
	}
}

/// The full weeks of a month grid, leading and trailing with adjacent
/// months' dates so every week is complete.
fn month_weeks(month: CalendarMonth) -> Vec<Vec<DateValue>> {
	let first = DateValue::new(month.year, month.month, 1);
	let leading = first.weekday() as i32;

	let start = first.add(leptos_aria_date::DateDuration::days(-leading));
	let length = leptos_aria_date::days_in_month(month.year, month.month);
	let total_days = leading + length as i32;
	let weeks = (total_days + 6) / 7;

	(0..weeks)
		.map(|week| {
			(0..7)
				.map(|day| start.add(leptos_aria_date::DateDuration::days(week * 7 + day)))
				.collect()
		})
		.collect()
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
	use leptos::mount::mount_to_body;
	use wasm_bindgen_test::*;

	use super::*;

	wasm_bindgen_test_configure!(run_in_browser);

	#[wasm_bindgen_test]
	async fn calendar_renders_the_month_grid() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! { <Calendar default_value=DateValue::new(2026, 10, 18) /> }
		});

		let grid = document().query_selector("[role='grid']").unwrap();

		assert!(grid.is_some(), "the calendar is a grid");

		let selected = document()
			.query_selector(".leptos-aria-calendar__cell--selected")
			.unwrap();

		assert!(selected.is_some(), "the selected date is marked");
	}

	#[wasm_bindgen_test]
	async fn calendar_appends_the_caller_class() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! { <Calendar class="custom-calendar" default_value=DateValue::new(2026, 10, 18) /> }
		});

		let calendar = document()
			.query_selector(".leptos-aria-calendar.custom-calendar")
			.unwrap()
			.expect("the caller class sits on the calendar root");

		assert_eq!(
			calendar.get_attribute("class").as_deref(),
			Some("leptos-aria-calendar custom-calendar")
		);
	}
}
