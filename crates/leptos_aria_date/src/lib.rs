#![doc(html_logo_url = "https://ifiokjr.github.io/leptos_aria/logo.svg")]
#![doc(html_favicon_url = "https://ifiokjr.github.io/leptos_aria/favicon.ico")]

/// A calendar date in the proleptic Gregorian calendar, the port of the
/// upstream `CalendarDate` value type.
///
/// Dates are plain year/month/day triples; arithmetic goes through
/// [`DateDuration`] additions so month ends and leap years are respected.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DateValue {
	/// The full year, negative before the common era.
	pub year: i32,

	/// The month, `1` through `12`.
	pub month: u32,

	/// The day of the month, `1` through the month's length.
	pub day: u32,
}

/// A distance between dates in calendar units.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DateDuration {
	/// Years to shift.
	pub years: i32,

	/// Months to shift.
	pub months: i32,

	/// Days to shift.
	pub days: i32,
}

impl DateValue {
	/// A date, clamping the day to the month's length.
	pub fn new(year: i32, month: u32, day: u32) -> Self {
		let month = month.clamp(1, 12);
		let day = day.min(days_in_month(year, month));

		Self { year, month, day }
	}

	/// The number of days in a month, respecting leap years.
	pub fn month_length(&self) -> u32 {
		days_in_month(self.year, self.month)
	}

	/// The day of the week, `0` for Sunday through `6` for Saturday.
	pub fn weekday(&self) -> u32 {
		// Days since 1970-01-01, a Thursday (`4`): whole years, the leap
		// days before this year (477 of them fall before 1970), and the days
		// of this year before this month.
		let leap_days = |year: i32| {
			let before = year - 1;

			before / LEAP_CYCLE - before / CENTURY + before / FOUR_CENTURIES - LEAPS_BEFORE_1970
		};

		let month_days: u32 = (1..self.month).map(|m| days_in_month(self.year, m)).sum();

		let days = i64::from(self.year - EPOCH_YEAR) * 365
			+ i64::from(leap_days(self.year))
			+ i64::from(month_days)
			+ i64::from(self.day)
			- 1;

		(days + EPOCH_WEEKDAY).rem_euclid(DAYS_PER_WEEK) as u32
	}

	/// The date shifted by a duration, clamping the day to the month's
	/// length after the month and year shifts.
	#[must_use]
	pub fn add(&self, duration: DateDuration) -> Self {
		let total_months =
			(self.year * 12 + self.month as i32 - 1) + duration.years * 12 + duration.months;

		let year = total_months.div_euclid(12);
		let month = total_months.rem_euclid(12) as u32 + 1;
		let clamped = DateValue::new(year, month, self.day);

		if duration.days == 0 {
			return clamped;
		}

		if duration.days > 0 {
			let (year, month, day) =
				advance_days(year, month, clamped.day, i64::from(duration.days));

			return DateValue::new(year, month, day);
		}

		let (year, month, day) = retreat_days(year, month, clamped.day, -i64::from(duration.days));

		DateValue::new(year, month, day)
	}
}

impl DateDuration {
	/// An empty duration.
	pub fn new() -> Self {
		Self::default()
	}

	/// A duration of the given number of months.
	pub fn months(count: i32) -> Self {
		Self::default().with_months(count)
	}

	/// A duration of the given number of days.
	pub fn days(count: i32) -> Self {
		Self::default().with_days(count)
	}

	/// The duration with other days.
	#[must_use]
	pub fn with_days(mut self, days: i32) -> Self {
		self.days = days;
		self
	}

	/// The duration with other months.
	#[must_use]
	pub fn with_months(mut self, months: i32) -> Self {
		self.months = months;
		self
	}

	/// The duration with other years.
	#[must_use]
	pub fn with_years(mut self, years: i32) -> Self {
		self.years = years;
		self
	}
}

/// The number of months in a year.
pub const MONTHS_PER_YEAR: i32 = 12;

/// The Gregorian leap cycle: every fourth year is leap, except centuries not
/// divisible by four hundred.
const LEAP_CYCLE: i32 = 4;
const CENTURY: i32 = 100;
const FOUR_CENTURIES: i32 = 400;

/// The month numbers of the Gregorian year.
pub const JANUARY: u32 = 1;
pub const FEBRUARY: u32 = 2;
pub const MARCH: u32 = 3;
pub const APRIL: u32 = 4;
pub const MAY: u32 = 5;
pub const JUNE: u32 = 6;
pub const JULY: u32 = 7;
pub const AUGUST: u32 = 8;
pub const SEPTEMBER: u32 = 9;
pub const OCTOBER: u32 = 10;
pub const NOVEMBER: u32 = 11;
pub const DECEMBER: u32 = 12;

/// The length of February in a common year.
const FEBRUARY_LENGTH: u32 = 28;

/// The number of leap days before 1970 in the Gregorian calendar.
const LEAPS_BEFORE_1970: i32 = 477;

/// The reference epoch, 1970-01-01, a Thursday.
const EPOCH_YEAR: i32 = 1970;
const EPOCH_WEEKDAY: i64 = 4;

/// Seconds-in-a-day style helpers for weekday arithmetic.
const DAYS_PER_WEEK: i64 = 7;

/// Whether a year is a leap year in the Gregorian calendar.
pub fn is_leap_year(year: i32) -> bool {
	(year % LEAP_CYCLE == 0 && year % CENTURY != 0) || year % FOUR_CENTURIES == 0
}

/// The number of days in a month.
pub fn days_in_month(year: i32, month: u32) -> u32 {
	match month {
		JANUARY | MARCH | MAY | JULY | AUGUST | OCTOBER | DECEMBER => 31,
		APRIL | JUNE | SEPTEMBER | NOVEMBER => 30,
		FEBRUARY if is_leap_year(year) => 29,
		_ => FEBRUARY_LENGTH,
	}
}

/// Walk forward across month ends.
fn advance_days(mut year: i32, mut month: u32, mut day: u32, mut days: i64) -> (i32, u32, u32) {
	while days > 0 {
		let length = i64::from(days_in_month(year, month));

		if i64::from(day) + days > length {
			days -= length + 1 - i64::from(day);
			month += 1;

			if month > 12 {
				month = 1;
				year += 1;
			}

			day = 1;
		} else {
			day += days as u32;
			days = 0;
		}
	}

	(year, month, day)
}

/// Walk backward across month starts.
fn retreat_days(mut year: i32, mut month: u32, mut day: u32, mut days: i64) -> (i32, u32, u32) {
	while days > 0 {
		if i64::from(day) > days {
			day -= days as u32;
			days = 0;
		} else {
			days -= i64::from(day);
			month -= 1;

			if month == 0 {
				month = 12;
				year -= 1;
			}

			day = days_in_month(year, month);
		}
	}

	(year, month, day)
}
