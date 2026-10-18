use web_sys::DomRect;
use web_sys::DragEvent;
use web_sys::Element;
use web_sys::MouseEvent;
use web_sys::PointerEvent;

/// A rectangle in client (viewport) coordinates.
#[derive(Debug, Clone, Copy, Default)]
pub struct Rect {
	pub top: f64,
	pub right: f64,
	pub bottom: f64,
	pub left: f64,
}

/// Anything that occupies an area of the viewport, for hit testing during a
/// press interaction.
pub trait GetRects {
	fn get_rects(&self) -> Vec<Rect>;
}

impl GetRects for MouseEvent {
	fn get_rects(&self) -> Vec<Rect> {
		vec![point_rect(self.client_x(), self.client_y())]
	}
}

impl GetRects for PointerEvent {
	fn get_rects(&self) -> Vec<Rect> {
		let offset_x = self.width();
		let offset_y = self.height();

		vec![Rect {
			top: (self.client_y() - offset_y).into(),
			right: (self.client_x() + offset_x).into(),
			bottom: (self.client_y() + offset_y).into(),
			left: (self.client_x() - offset_x).into(),
		}]
	}
}

impl GetRects for DragEvent {
	fn get_rects(&self) -> Vec<Rect> {
		vec![point_rect(self.client_x(), self.client_y())]
	}
}

impl GetRects for web_sys::TouchEvent {
	fn get_rects(&self) -> Vec<Rect> {
		let touches = self.touches();
		let mut rects = Vec::with_capacity(touches.length() as usize);

		for index in 0..touches.length() {
			let Some(touch) = touches.item(index) else {
				continue;
			};

			let offset_x = touch.radius_x();
			let offset_y = touch.radius_y();

			rects.push(Rect {
				top: (touch.client_y() - offset_y).into(),
				right: (touch.client_x() + offset_x).into(),
				bottom: (touch.client_y() + offset_y).into(),
				left: (touch.client_x() - offset_x).into(),
			});
		}

		rects
	}
}

fn point_rect(x: i32, y: i32) -> Rect {
	Rect {
		top: f64::from(y),
		right: f64::from(x),
		bottom: f64::from(y),
		left: f64::from(x),
	}
}

/// Whether any of the point's rectangles overlaps the target's bounding box.
pub(crate) fn is_above_target(point: &impl GetRects, target: &Element) -> bool {
	let rect = target.get_bounding_client_rect();
	let point_rects = point.get_rects();

	are_rectangles_overlapping(&rect, &point_rects)
}

fn are_rectangles_overlapping(dom_rect: &DomRect, rects: &[Rect]) -> bool {
	rects
		.iter()
		.any(|rect| overlaps_on_x_axis(dom_rect, rect) && overlaps_on_y_axis(dom_rect, rect))
}

fn overlaps_on_x_axis(dom_rect: &DomRect, rect: &Rect) -> bool {
	dom_rect.left() <= rect.right && dom_rect.right() >= rect.left
}

fn overlaps_on_y_axis(dom_rect: &DomRect, rect: &Rect) -> bool {
	dom_rect.top() <= rect.bottom && dom_rect.bottom() >= rect.top
}
