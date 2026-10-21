use crate::use_id;

/// The props for the label element, if the field has a label.
#[derive(Clone, Debug, Default)]
pub struct LabelProps {
	/// The id to set on the label element.
	pub id: String,

	/// The `for` attribute to set on the label element, pointing at the field.
	pub r#for: String,
}

/// The accessibility props to spread onto the labeled field element.
#[derive(Clone, Debug, Default)]
pub struct FieldProps {
	/// The id to set on the field element.
	pub id: String,

	/// The `aria-label` attribute when no visible label element exists.
	pub aria_label: Option<String>,

	/// The `aria-labelledby` attribute wiring the label element to the field.
	pub aria_labelledby: Option<String>,

	/// The `aria-describedby` attribute wiring the description element to the
	/// field.
	pub aria_describedby: Option<String>,

	/// The `aria-details` attribute wiring the details element to the field.
	pub aria_details: Option<String>,
}

/// The props accepted by [`use_label`].
#[derive(Clone, Debug, Default)]
pub struct UseLabelProps {
	/// The content to display as the label.
	pub label: Option<String>,

	/// The id to set on the label element. One is generated when missing.
	pub label_id: Option<String>,

	/// The id to set on the field element. One is generated when missing.
	pub id: Option<String>,

	/// The element type of the field: real form controls can be labeled by a
	/// `<label for>` element, other elements fall back to `aria-labelledby`.
	/// Defaults to `"label"`.
	pub label_element_type: LabelElementType,

	/// The `aria-labelledby` ids of extra labeling elements.
	pub aria_labelledby: Option<String>,

	/// The `aria-label` for when there is no visible label at all.
	pub aria_label: Option<String>,

	/// The `aria-describedby` ids of description elements.
	pub aria_describedby: Option<String>,

	/// The `aria-details` ids of detail elements.
	pub aria_details: Option<String>,
}

/// Whether a `<label for>` element can label the field.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum LabelElementType {
	/// A real form control (`input`, `select`, `textarea`): labelable.
	#[default]
	Label,
	/// Any other element: labeled through `aria-labelledby`.
	Span,
}

/// The result of [`use_label`]: props for the label and the labeled field.
#[derive(Clone, Debug, Default)]
pub struct UseLabelResult {
	pub label_props: LabelProps,
	pub field_props: FieldProps,
}

/// Provides the accessibility wiring between a visible label and a field.
///
/// A labelable field gets a `<label for>` relationship; anything else is
/// wired through `aria-labelledby` instead.
#[allow(clippy::needless_pass_by_value)]
pub fn use_label(props: UseLabelProps) -> UseLabelResult {
	let id = props.id.clone().unwrap_or_else(use_id);

	let label_props = match props.label {
		Some(_) => {
			LabelProps {
				id: props.label_id.clone().unwrap_or_else(use_id),
				r#for: id.clone(),
			}
		}
		None => LabelProps::default(),
	};

	let field_props = build_field_props(&props, &label_props, &id);

	UseLabelResult {
		label_props,
		field_props,
	}
}

fn build_field_props(props: &UseLabelProps, label_props: &LabelProps, id: &str) -> FieldProps {
	let aria_labelledby = preferred_aria_labelledby(props, label_props);

	FieldProps {
		id: id.to_string(),
		aria_label: props.aria_label.clone(),
		aria_labelledby,
		aria_describedby: props.aria_describedby.clone(),
		aria_details: props.aria_details.clone(),
	}
}

fn preferred_aria_labelledby(props: &UseLabelProps, label_props: &LabelProps) -> Option<String> {
	// A <label for> element labels the field itself, so aria-labelledby only
	// carries the extra labeling ids; non-labelable fields include the label
	// element id as well. Without a label there is no id to include.
	let labelable = matches!(props.label_element_type, LabelElementType::Label);

	if labelable || label_props.id.is_empty() {
		return non_empty(props.aria_labelledby.as_deref());
	}

	match props.aria_labelledby.as_deref() {
		Some("") | None => Some(label_props.id.clone()),
		Some(extra) => Some(format!("{} {extra}", label_props.id)),
	}
}

fn non_empty(id: Option<&str>) -> Option<String> {
	id.filter(|id| !id.is_empty()).map(str::to_string)
}

/// The props accepted by [`use_labels`]: either a visible label or the
/// aria labeling attributes, never both.
#[derive(Clone, Debug, Default)]
pub struct UseLabelsProps {
	/// The content to display as the label.
	pub label: Option<String>,

	/// The `aria-label` for when there is no visible label at all.
	pub aria_label: Option<String>,

	/// The `aria-labelledby` ids of labeling elements.
	pub aria_labelledby: Option<String>,

	/// The `aria-describedby` ids of description elements.
	pub aria_describedby: Option<String>,

	/// The `aria-details` ids of detail elements.
	pub aria_details: Option<String>,

	/// The element type of the field.
	pub label_element_type: LabelElementType,
}

/// Provides labeling wiring where a visible label replaces `aria-label`.
///
/// If a visible label is provided, `aria-label` and `aria-labelledby` are
/// ignored; otherwise the aria attributes are passed through.
#[allow(clippy::needless_pass_by_value)]
pub fn use_labels(props: UseLabelsProps) -> UseLabelResult {
	let has_label = props.label.is_some();

	let resolved = if has_label {
		UseLabelProps {
			label: props.label.clone(),
			label_element_type: props.label_element_type,
			aria_labelledby: props.aria_labelledby.clone(),
			aria_describedby: props.aria_describedby.clone(),
			aria_details: props.aria_details.clone(),
			..Default::default()
		}
	} else {
		UseLabelProps {
			aria_label: props.aria_label.clone(),
			aria_labelledby: props.aria_labelledby.clone(),
			aria_describedby: props.aria_describedby.clone(),
			aria_details: props.aria_details.clone(),
			label_element_type: props.label_element_type,
			..Default::default()
		}
	};

	use_label(resolved)
}

#[cfg(test)]
mod tests {
	use super::*;

	fn labelledby(
		label: Option<&str>,
		element: LabelElementType,
		extra: Option<&str>,
	) -> Option<String> {
		let result = use_labels(UseLabelsProps {
			label: label.map(str::to_string),
			aria_labelledby: extra.map(str::to_string),
			label_element_type: element,
			..Default::default()
		});

		result.field_props.aria_labelledby
	}

	#[test]
	fn a_label_for_names_a_labelable_field_without_aria_labelledby() {
		assert_eq!(
			labelledby(Some("Name"), LabelElementType::Label, None),
			None
		);
		assert_eq!(
			labelledby(Some("Name"), LabelElementType::Label, Some("hint")).as_deref(),
			Some("hint")
		);
	}

	#[test]
	fn a_non_labelable_field_is_named_by_its_label_id() {
		let result = use_labels(UseLabelsProps {
			label: Some("Code".to_string()),
			aria_labelledby: Some("hint".to_string()),
			label_element_type: LabelElementType::Span,
			..Default::default()
		});

		assert_eq!(
			result.field_props.aria_labelledby,
			Some(format!("{} hint", result.label_props.id))
		);
	}

	#[test]
	fn a_non_labelable_field_without_a_label_has_no_empty_aria_labelledby() {
		assert_eq!(labelledby(None, LabelElementType::Span, None), None);
		assert_eq!(labelledby(None, LabelElementType::Span, Some("")), None);
		assert_eq!(
			labelledby(None, LabelElementType::Span, Some("heading")).as_deref(),
			Some("heading")
		);
	}
}
