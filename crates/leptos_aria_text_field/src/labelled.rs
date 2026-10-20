//! A label followed by its field, with spread attributes reaching only the
//! field.
//!
//! A plain `(label, field)` fragment hands every `attr:` spread to both
//! elements, so `attr:data-slot="input"` on a [`TextField`](crate::TextField)
//! would land on its `<label>` too. Upstream `useTextField` merges DOM props
//! into the input's props alone; [`LabelledField`] gives the components the
//! same contract.

use leptos::tachys::html::attribute::Attribute;
use leptos::tachys::html::attribute::any_attribute::AnyAttribute;
use leptos::tachys::hydration::Cursor;
use leptos::tachys::view::Position;
use leptos::tachys::view::PositionState;
use leptos::tachys::view::Render;
use leptos::tachys::view::RenderHtml;
use leptos::tachys::view::add_attr::AddAnyAttr;

/// Renders `label` then `field`; attributes spread onto it go to `field`.
pub(crate) struct LabelledField<L, F> {
	pub label: L,
	pub field: F,
}

impl<L: Render, F: Render> Render for LabelledField<L, F> {
	type State = (L::State, F::State);

	fn build(self) -> Self::State {
		(self.label.build(), self.field.build())
	}

	fn rebuild(self, state: &mut Self::State) {
		self.label.rebuild(&mut state.0);
		self.field.rebuild(&mut state.1);
	}
}

impl<L: RenderHtml, F: RenderHtml> RenderHtml for LabelledField<L, F> {
	type AsyncOutput = LabelledField<L::AsyncOutput, F::AsyncOutput>;
	type Owned = LabelledField<L::Owned, F::Owned>;

	const MIN_LENGTH: usize = L::MIN_LENGTH + F::MIN_LENGTH;

	fn html_len(&self) -> usize {
		self.label.html_len() + self.field.html_len()
	}

	fn dry_resolve(&mut self) {
		self.label.dry_resolve();
		self.field.dry_resolve();
	}

	async fn resolve(self) -> Self::AsyncOutput {
		LabelledField {
			label: self.label.resolve().await,
			field: self.field.resolve().await,
		}
	}

	fn to_html_with_buf(
		self,
		buf: &mut String,
		position: &mut Position,
		escape: bool,
		mark_branches: bool,
		extra_attrs: Vec<AnyAttribute>,
	) {
		self.label
			.to_html_with_buf(buf, position, escape, mark_branches, Vec::new());
		self.field
			.to_html_with_buf(buf, position, escape, mark_branches, extra_attrs);
	}

	fn hydrate<const FROM_SERVER: bool>(
		self,
		cursor: &Cursor,
		position: &PositionState,
	) -> Self::State {
		(
			self.label.hydrate::<FROM_SERVER>(cursor, position),
			self.field.hydrate::<FROM_SERVER>(cursor, position),
		)
	}

	fn into_owned(self) -> Self::Owned {
		LabelledField {
			label: self.label.into_owned(),
			field: self.field.into_owned(),
		}
	}
}

impl<L: RenderHtml, F: RenderHtml> AddAnyAttr for LabelledField<L, F> {
	type Output<SomeNewAttr: Attribute> = LabelledField<L, F::Output<SomeNewAttr>>;

	fn add_any_attr<NewAttr: Attribute>(self, attr: NewAttr) -> Self::Output<NewAttr>
	where
		Self::Output<NewAttr>: RenderHtml,
	{
		LabelledField {
			label: self.label,
			field: self.field.add_any_attr(attr),
		}
	}
}
