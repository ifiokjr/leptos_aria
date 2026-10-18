/// A color in the sRGB color space.
///
/// The port of the upstream `Color` value type over its parsable string
/// forms: hex, `rgb()`, `rgba()`, `hsl()`, and `hsla()`.
#[derive(Clone, Debug, PartialEq)]
pub struct Color {
	/// The red channel, `0.0` through `1.0`.
	pub red: f64,

	/// The green channel, `0.0` through `1.0`.
	pub green: f64,

	/// The blue channel, `0.0` through `1.0`.
	pub blue: f64,

	/// The alpha channel, `0.0` through `1.0`.
	pub alpha: f64,
}

impl Color {
	/// A color from channels already in range.
	pub fn new(red: f64, green: f64, blue: f64, alpha: f64) -> Self {
		Self {
			red: red.clamp(0.0, 1.0),
			green: green.clamp(0.0, 1.0),
			blue: blue.clamp(0.0, 1.0),
			alpha: alpha.clamp(0.0, 1.0),
		}
	}

	/// Parse a CSS color string.
	pub fn parse(value: &str) -> Option<Self> {
		let value = value.trim();

		if let Some(hex) = value.strip_prefix('#') {
			return parse_hex(hex);
		}

		let function = value.split('(').next()?.trim().to_lowercase();
		let arguments: Vec<f64> = value
			.split('(')
			.nth(1)?
			.strip_suffix(')')?
			.split([',', ' ', '/'])
			.filter(|part| !part.is_empty())
			.map(|part| parse_number(part.trim()))
			.collect::<Option<_>>()?;

		match (function.as_str(), arguments.as_slice()) {
			("rgb", [red, green, blue]) => {
				Some(Self::new(red / 255.0, green / 255.0, blue / 255.0, 1.0))
			}
			("rgba", [red, green, blue, alpha]) => {
				Some(Self::new(red / 255.0, green / 255.0, blue / 255.0, *alpha))
			}
			("hsl", [hue, saturation, lightness]) => from_hsl(*hue, *saturation, *lightness, 1.0),
			("hsla", [hue, saturation, lightness, alpha]) => {
				from_hsl(*hue, *saturation, *lightness, *alpha)
			}
			_ => None,
		}
	}

	/// The hue in degrees, `0.0` through `360.0`.
	pub fn hue(&self) -> f64 {
		let (hue, ..) = to_hsl(self);

		hue
	}

	/// The saturation as a fraction of `1.0`.
	pub fn saturation(&self) -> f64 {
		let (_, saturation, _) = to_hsl(self);

		saturation
	}

	/// The lightness as a fraction of `1.0`.
	pub fn lightness(&self) -> f64 {
		let (_, _, lightness) = to_hsl(self);

		lightness
	}

	/// The color shifted to another hue in degrees.
	#[must_use]
	pub fn with_hue(&self, hue: f64) -> Self {
		let (_, saturation, lightness) = to_hsl(self);

		from_hsl(hue, saturation, lightness, self.alpha).unwrap_or_else(|| self.clone())
	}

	/// The color with another saturation.
	#[must_use]
	pub fn with_saturation(&self, saturation: f64) -> Self {
		let (hue, _, lightness) = to_hsl(self);

		from_hsl(hue, saturation.clamp(0.0, 1.0), lightness, self.alpha)
			.unwrap_or_else(|| self.clone())
	}

	/// The color with another lightness.
	#[must_use]
	pub fn with_lightness(&self, lightness: f64) -> Self {
		let (hue, saturation, _) = to_hsl(self);

		from_hsl(hue, saturation, lightness.clamp(0.0, 1.0), self.alpha)
			.unwrap_or_else(|| self.clone())
	}

	/// The `#rrggbb` form.
	pub fn to_hex(&self) -> String {
		let channel = |value: f64| format!("{:02x}", (value * 255.0).round() as u8);

		format!(
			"#{}{}{}",
			channel(self.red),
			channel(self.green),
			channel(self.blue)
		)
	}

	/// The `rgba()` form.
	pub fn to_rgb_string(&self) -> String {
		let channel = |value: f64| (value * 255.0).round() as u8;

		format!(
			"rgba({}, {}, {}, {})",
			channel(self.red),
			channel(self.green),
			channel(self.blue),
			round2(self.alpha)
		)
	}
}

/// Parse a number with an optional `%` suffix.
fn parse_number(part: &str) -> Option<f64> {
	if let Some(percent) = part.strip_suffix('%') {
		return percent
			.trim()
			.parse::<f64>()
			.ok()
			.map(|value| value / 100.0);
	}

	part.parse::<f64>().ok()
}

/// Parse `rgb`, `rrggbb`, `rgba`, or `rrggbbaa` hex digits.
fn parse_hex(hex: &str) -> Option<Color> {
	let channel = |digits: &str| {
		u8::from_str_radix(digits, 16)
			.ok()
			.map(|value| f64::from(value) / 255.0)
	};

	// The short forms expand each digit: `f` becomes `ff`.
	match (hex.len(), hex) {
		(3, _) => {
			Some(Color::new(
				channel(&hex[0..1].repeat(2))?,
				channel(&hex[1..2].repeat(2))?,
				channel(&hex[2..3].repeat(2))?,
				1.0,
			))
		}
		(4, _) => {
			Some(Color::new(
				channel(&hex[0..1].repeat(2))?,
				channel(&hex[1..2].repeat(2))?,
				channel(&hex[2..3].repeat(2))?,
				channel(&hex[3..4].repeat(2))?,
			))
		}
		(6, _) => {
			Some(Color::new(
				channel(&hex[0..2])?,
				channel(&hex[2..4])?,
				channel(&hex[4..6])?,
				1.0,
			))
		}
		(8, _) => {
			Some(Color::new(
				channel(&hex[0..2])?,
				channel(&hex[2..4])?,
				channel(&hex[4..6])?,
				channel(&hex[6..8])?,
			))
		}
		_ => None,
	}
}

/// Build an RGB color from hue in degrees and fractional saturation and
/// lightness.
fn from_hsl(hue: f64, saturation: f64, lightness: f64, alpha: f64) -> Option<Color> {
	let hue = ((hue % 360.0) + 360.0) % 360.0 / 360.0;

	if !(0.0..=1.0).contains(&saturation) || !(0.0..=1.0).contains(&lightness) {
		return None;
	}

	let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
	let sector = hue * 6.0;
	let secondary = chroma * (1.0 - (sector % 2.0 - 1.0).abs());
	let lightness_offset = lightness - chroma / 2.0;

	let (red, green, blue) = match sector.floor() as u32 % 6 {
		0 => (chroma, secondary, 0.0),
		1 => (secondary, chroma, 0.0),
		2 => (0.0, chroma, secondary),
		3 => (0.0, secondary, chroma),
		4 => (secondary, 0.0, chroma),
		_ => (chroma, 0.0, secondary),
	};

	Some(Color::new(
		red + lightness_offset,
		green + lightness_offset,
		blue + lightness_offset,
		alpha,
	))
}

/// Decompose a color into hue in degrees and fractional saturation and
/// lightness.
fn to_hsl(color: &Color) -> (f64, f64, f64) {
	let max = color.red.max(color.green).max(color.blue);
	let min = color.red.min(color.green).min(color.blue);
	let chroma = max - min;
	let lightness = f64::midpoint(max, min);

	if chroma == 0.0 {
		return (0.0, 0.0, lightness);
	}

	let saturation = chroma / (1.0 - (2.0 * lightness - 1.0).abs());

	let hue = if (max - color.red).abs() < f64::EPSILON {
		((color.green - color.blue) / chroma) % 6.0
	} else if (max - color.green).abs() < f64::EPSILON {
		(color.blue - color.red) / chroma + 2.0
	} else {
		(color.red - color.green) / chroma + 4.0
	};

	((hue * 60.0 + 360.0) % 360.0, saturation, lightness)
}

fn round2(value: f64) -> f64 {
	(value * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parses_hex_forms() {
		assert_eq!(Color::parse("#fff"), Some(Color::new(1.0, 1.0, 1.0, 1.0)));
		assert_eq!(
			Color::parse("#ff0000"),
			Some(Color::new(1.0, 0.0, 0.0, 1.0))
		);
		let alpha = Color::parse("#00ff0080").map(|c| c.alpha).unwrap();

		assert!((alpha - 128.0 / 255.0).abs() < 1e-12);
	}

	#[test]
	fn parses_function_forms() {
		assert_eq!(
			Color::parse("rgb(255, 0, 0)"),
			Some(Color::new(1.0, 0.0, 0.0, 1.0))
		);

		assert_eq!(
			Color::parse("hsl(120, 100%, 50%)"),
			Color::parse("rgb(0, 255, 0)")
		);
	}

	#[test]
	fn hue_and_saturation_round_trip() {
		let color = Color::parse("hsl(240, 100%, 50%)").unwrap();

		assert!((color.hue() - 240.0).abs() < 0.5);
		assert!((color.saturation() - 1.0).abs() < 1e-9);
		assert!((color.lightness() - 0.5).abs() < 1e-9);

		let shifted = color.with_hue(0.0);

		assert!((shifted.hue() - 0.0).abs() < 0.5, "hue shifts to red");
	}
}
