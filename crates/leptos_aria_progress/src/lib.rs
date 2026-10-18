use leptos::prelude::*;
use leptos_aria_utils::LabelElementType;
use leptos_aria_utils::UseLabelsProps;
use leptos_aria_utils::use_labels;

/// The props shared by [`ProgressBar`] and [`Meter`].
#[derive(Clone, Debug, Default)]
pub struct ProgressBaseProps {
	/// The current value of the progress, `0` by default.
	pub value: Option<Signal<f64>>,

	/// The smallest value allowed, `0` by default.
	pub min_value: Option<f64>,

	/// The largest value allowed, `100` by default.
	pub max_value: Option<f64>,

	/// The content to display as the label.
	pub label: Option<String>,

	/// The id of the element labeling the progress bar, when the label is
	/// rendered by the owner.
	pub aria_labelledby: Option<String>,

	/// The aria-label when there is no visible label.
	pub aria_label: Option<String>,

	/// The content to display as the value's label (e.g. "1 of 4").
	pub value_label: Option<String>,
}

/// The result of [`use_progress_bar`].
#[derive(Clone, Debug, Default)]
pub struct ProgressBarAria {
	/// The resolved aria props for the progress bar element.
	pub role: &'static str,

	/// The current value, or `None` while indeterminate.
	pub aria_value_now: Option<f64>,

	/// The minimum value.
	pub aria_value_min: f64,

	/// The maximum value.
	pub aria_value_max: f64,

	/// The human readable value text, or `None` while indeterminate.
	pub aria_value_text: Option<String>,

	/// The id of the element labeling the progress bar.
	pub aria_labelledby: Option<String>,

	/// The id to set on the progress bar element.
	pub id: String,
}

/// The fraction of the range the value has covered, between `0.0` and `1.0`.
pub fn progress_percentage(props: &ProgressBaseProps) -> f64 {
	let value = clamp_value(props);
	let (min_value, max_value) = value_range(props);
	let range = max_value - min_value;

	if range == 0.0 {
		return 0.0;
	}

	(value - min_value) / range
}

/// Provides the accessibility implementation for a progress bar component.
///
/// Progress bars show either determinate or indeterminate progress of an
/// operation over time.
pub fn use_progress_bar(props: &ProgressBaseProps, is_indeterminate: bool) -> ProgressBarAria {
	let labels = use_labels(UseLabelsProps {
		label: props.label.clone(),
		aria_label: props.aria_label.clone(),
		aria_labelledby: props.aria_labelledby.clone(),
		label_element_type: LabelElementType::Span,
		..Default::default()
	});

	let (min_value, max_value) = value_range(props);
	let aria_value_now = (!is_indeterminate).then(|| clamp_value(props));
	let aria_value_text = if is_indeterminate {
		None
	} else {
		Some(
			props
				.value_label
				.clone()
				.unwrap_or_else(|| default_value_label(props)),
		)
	};

	ProgressBarAria {
		role: "progressbar",
		aria_value_now,
		aria_value_min: min_value,
		aria_value_max: max_value,
		aria_value_text,
		aria_labelledby: labels.field_props.aria_labelledby,
		id: labels.field_props.id,
	}
}

/// The result of [`use_meter`], a progress bar with a `meter` role.
pub fn use_meter(props: &ProgressBaseProps) -> ProgressBarAria {
	let mut aria = use_progress_bar(props, false);
	aria.role = "meter";
	aria
}

fn value_range(props: &ProgressBaseProps) -> (f64, f64) {
	(
		props.min_value.unwrap_or(0.0),
		props.max_value.unwrap_or(100.0),
	)
}

fn clamp_value(props: &ProgressBaseProps) -> f64 {
	let value = props.value.map_or(0.0, |value| value.get_untracked());
	let (min_value, max_value) = value_range(props);

	value.clamp(min_value, max_value)
}

/// The default value label formats the covered percentage.
///
/// The upstream implementation formats through `Intl.NumberFormat` with the
/// configured options; until the i18n formatting port, this renders a plain
/// whole-number percentage.
fn default_value_label(props: &ProgressBaseProps) -> String {
	let percentage = progress_percentage(props) * 100.0;

	format!("{}%", percentage.round() as u32)
}

/// Shows either determinate or indeterminate progress of an operation over
/// time.
#[component]
pub fn ProgressBar(
	/// The current value of the progress, `0` by default.
	#[prop(optional, into)]
	value: Option<Signal<f64>>,
	/// The smallest value allowed, `0` by default.
	#[prop(optional)]
	min_value: Option<f64>,
	/// The largest value allowed, `100` by default.
	#[prop(optional)]
	max_value: Option<f64>,
	/// The content to display as the label.
	#[prop(optional)]
	label: Option<String>,
	/// Whether presentation is indeterminate when progress isn't known.
	#[prop(optional, into)]
	is_indeterminate: Signal<bool>,
	/// The content to display as the value's label (e.g. "1 of 4").
	#[prop(optional)]
	value_label: Option<String>,
	#[prop(optional)] children: Option<Children>,
) -> impl IntoView {
	let props = ProgressBaseProps {
		value,
		min_value,
		max_value,
		label,
		aria_labelledby: None,
		aria_label: None,
		value_label,
	};

	let aria = use_progress_bar(&props, is_indeterminate.get_untracked());
	let percentage = progress_percentage(&props);

	view! {
		<div
			id=aria.id.clone()
			role=aria.role
			aria-valuenow=aria.aria_value_now.map(|value| value.to_string())
			aria-valuemin=aria.aria_value_min.to_string()
			aria-valuemax=aria.aria_value_max.to_string()
			aria-valuetext=aria.aria_value_text.clone()
			aria-labelledby=aria.aria_labelledby.clone()
		>
			<span class="leptos-aria-progress-bar__track">
				<span
					class="leptos-aria-progress-bar__fill"
					style=move || {
						if is_indeterminate.get() {
							String::new()
						} else {
							format!("width: {:.1}%;", percentage * 100.0)
						}
					}
				>
					{children.map(|children| children())}
				</span>
			</span>
		</div>
	}
}

/// A meter represents a quantity within a known range, or a fractional value.
#[component]
pub fn Meter(
	/// The current value of the meter, `0` by default.
	#[prop(optional, into)]
	value: Option<Signal<f64>>,
	/// The smallest value allowed, `0` by default.
	#[prop(optional)]
	min_value: Option<f64>,
	/// The largest value allowed, `100` by default.
	#[prop(optional)]
	max_value: Option<f64>,
	/// The content to display as the label.
	#[prop(optional)]
	label: Option<String>,
	/// The content to display as the value's label (e.g. "1 of 4").
	#[prop(optional)]
	value_label: Option<String>,
	#[prop(optional)] children: Option<Children>,
) -> impl IntoView {
	let props = ProgressBaseProps {
		value,
		min_value,
		max_value,
		label,
		aria_labelledby: None,
		aria_label: None,
		value_label,
	};

	let aria = use_meter(&props);
	let percentage = progress_percentage(&props);

	view! {
		<div
			id=aria.id.clone()
			role=aria.role
			aria-valuenow=aria.aria_value_now.map(|value| value.to_string())
			aria-valuemin=aria.aria_value_min.to_string()
			aria-valuemax=aria.aria_value_max.to_string()
			aria-valuetext=aria.aria_value_text.clone()
			aria-labelledby=aria.aria_labelledby.clone()
		>
			<span class="leptos-aria-meter__track">
				<span class="leptos-aria-meter__fill" style=format!("width: {:.1}%;", percentage * 100.0)>
					{children.map(|children| children())}
				</span>
			</span>
		</div>
	}
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
	use leptos::mount::mount_to_body;
	use wasm_bindgen_test::*;

	use super::*;

	wasm_bindgen_test_configure!(run_in_browser);

	#[wasm_bindgen_test]
	fn progress_bar_rends_determinate_aria() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! {
				<ProgressBar value=Signal::from(25.0) label="Loading".to_string() />
			}
		});

		let bar = document()
			.query_selector("[role='progressbar']")
			.unwrap()
			.unwrap();

		assert_eq!(bar.get_attribute("aria-valuenow").as_deref(), Some("25"));
		assert_eq!(bar.get_attribute("aria-valuemin").as_deref(), Some("0"));
		assert_eq!(bar.get_attribute("aria-valuemax").as_deref(), Some("100"));
		assert_eq!(bar.get_attribute("aria-valuetext").as_deref(), Some("25%"));
	}

	#[wasm_bindgen_test]
	fn meter_uses_the_meter_role() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! { <Meter value=Signal::from(64.0) label="Disk usage".to_string() /> }
		});

		let meter = document()
			.query_selector("[role='meter']")
			.unwrap()
			.unwrap();

		assert_eq!(meter.get_attribute("aria-valuenow").as_deref(), Some("64"));
	}
}
