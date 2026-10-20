#![doc(html_logo_url = "https://ifiokjr.github.io/leptos_aria/logo.svg")]
#![doc(html_favicon_url = "https://ifiokjr.github.io/leptos_aria/favicon.ico")]

use leptos::prelude::*;
use leptos_aria_utils::LabelElementType;
use leptos_aria_utils::UseLabelsProps;
use leptos_aria_utils::merge_class;
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

/// The value-dependent ARIA attributes and fill of a progress element.
#[derive(Clone, Debug, PartialEq)]
struct ProgressValues {
	now: Option<String>,
	text: Option<String>,
	fill_style: Option<String>,
}

/// Track the value and indeterminate state, recomputing the attributes that
/// depend on them.
fn use_progress_values(
	props: ProgressBaseProps,
	is_indeterminate: Signal<bool>,
) -> Memo<ProgressValues> {
	Memo::new(move |_| {
		// The shared helpers read the value untracked, so subscribe here.
		if let Some(value) = props.value {
			value.track();
		}

		if is_indeterminate.get() {
			return ProgressValues {
				now: None,
				text: None,
				fill_style: None,
			};
		}

		ProgressValues {
			now: Some(clamp_value(&props).to_string()),
			text: Some(
				props
					.value_label
					.clone()
					.unwrap_or_else(|| default_value_label(&props)),
			),
			fill_style: Some(format!(
				"width: {:.1}%;",
				progress_percentage(&props) * 100.0
			)),
		}
	})
}

/// The shared view of [`ProgressBar`] and [`Meter`]: an optional label, then
/// the root with its track and fill.
fn progress_view(
	props: ProgressBaseProps,
	aria: &ProgressBarAria,
	is_indeterminate: Signal<bool>,
	part_classes: (&'static str, &'static str),
	class: Signal<String>,
	children: Option<Children>,
) -> impl IntoView + use<> {
	let label = props.label.clone();
	let values = use_progress_values(props, is_indeterminate);
	let (track_class, fill_class) = part_classes;

	view! {
		{label
			.map(|label| {
				view! {
					<span id=aria.aria_labelledby.clone() class="leptos-aria-progress__label">
						{label}
					</span>
				}
			})}
		<div
			id=aria.id.clone()
			class=merge_class("", class)
			role=aria.role
			aria-valuenow=move || values.with(|values| values.now.clone())
			aria-valuemin=aria.aria_value_min.to_string()
			aria-valuemax=aria.aria_value_max.to_string()
			aria-valuetext=move || values.with(|values| values.text.clone())
			aria-labelledby=aria.aria_labelledby.clone()
		>
			<span class=track_class>
				<span class=fill_class style=move || values.with(|values| values.fill_style.clone())>
					{children.map(|children| children())}
				</span>
			</span>
		</div>
	}
}

/// Shows either determinate or indeterminate progress of an operation over
/// time.
///
/// The ARIA values and the fill width follow `value` and
/// `is_indeterminate`. A `label` renders in a `span` before the bar, which
/// the bar's `aria-labelledby` points at.
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
	/// Classes for the root element.
	#[prop(optional, into)]
	class: Signal<String>,
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

	progress_view(
		props,
		&aria,
		is_indeterminate,
		(
			"leptos-aria-progress-bar__track",
			"leptos-aria-progress-bar__fill",
		),
		class,
		children,
	)
}

/// A meter represents a quantity within a known range, or a fractional value.
///
/// The ARIA values and the fill width follow `value`.
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
	/// Classes for the root element.
	#[prop(optional, into)]
	class: Signal<String>,
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

	progress_view(
		props,
		&aria,
		Signal::from(false),
		("leptos-aria-meter__track", "leptos-aria-meter__fill"),
		class,
		children,
	)
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
				<ProgressBar value=Signal::from(25.0) label="Loading".to_string() class="progress-test-determinate" />
			}
		});

		let bar = document()
			.query_selector(".progress-test-determinate")
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
			view! { <Meter value=Signal::from(64.0) label="Disk usage".to_string() class="meter-test-role" /> }
		});

		let meter = document()
			.query_selector(".meter-test-role")
			.unwrap()
			.unwrap();

		assert_eq!(meter.get_attribute("aria-valuenow").as_deref(), Some("64"));
	}

	async fn tick() {
		let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
		wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
	}

	#[wasm_bindgen_test]
	async fn progress_bar_follows_its_value() {
		console_error_panic_hook::set_once();

		let value = RwSignal::new(25.0);

		mount_to_body(move || {
			view! { <ProgressBar value=value class="progress-test-reactive" /> }
		});

		let bar = document()
			.query_selector(".progress-test-reactive")
			.unwrap()
			.unwrap();
		let fill = bar
			.query_selector(".leptos-aria-progress-bar__fill")
			.unwrap()
			.unwrap();

		value.set(75.0);
		tick().await;

		assert_eq!(bar.get_attribute("aria-valuenow").as_deref(), Some("75"));
		assert_eq!(bar.get_attribute("aria-valuetext").as_deref(), Some("75%"));
		assert_eq!(
			fill.get_attribute("style").as_deref(),
			Some("width: 75.0%;")
		);
	}

	#[wasm_bindgen_test]
	async fn progress_bar_drops_its_value_while_indeterminate() {
		console_error_panic_hook::set_once();

		let is_indeterminate = RwSignal::new(false);

		mount_to_body(move || {
			view! {
				<ProgressBar
					value=Signal::from(40.0)
					is_indeterminate=is_indeterminate
					class="progress-test-indeterminate"
				/>
			}
		});

		let bar = document()
			.query_selector(".progress-test-indeterminate")
			.unwrap()
			.unwrap();

		is_indeterminate.set(true);
		tick().await;

		assert!(!bar.has_attribute("aria-valuenow"));
		assert!(!bar.has_attribute("aria-valuetext"));
	}

	#[wasm_bindgen_test]
	fn progress_bar_renders_the_label_it_points_at() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! {
				<ProgressBar value=Signal::from(10.0) label="Uploading".to_string() class="progress-test-label" />
			}
		});

		let bar = document()
			.query_selector(".progress-test-label")
			.unwrap()
			.unwrap();
		let label_id = bar
			.get_attribute("aria-labelledby")
			.expect("a labelled bar points at its label");
		let label = document()
			.get_element_by_id(&label_id)
			.expect("the label element is rendered");

		assert_eq!(label.text_content().as_deref(), Some("Uploading"));
	}

	#[wasm_bindgen_test]
	async fn meter_follows_its_value() {
		console_error_panic_hook::set_once();

		let value = RwSignal::new(10.0);

		mount_to_body(move || view! { <Meter value=value class="meter-test-reactive" /> });

		let meter = document()
			.query_selector(".meter-test-reactive")
			.unwrap()
			.unwrap();

		value.set(90.0);
		tick().await;

		assert_eq!(meter.get_attribute("aria-valuenow").as_deref(), Some("90"));
	}
}
