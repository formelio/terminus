// Vendored in from fhir_model (https://github.com/FlixCoder/fhir-sdk/blob/main/crates/fhir-model/src/date_time.rs)

use ::serde::{Deserialize, Serialize};
use ::std::cmp::Ordering;
use ::std::str::FromStr;
use ::time::OffsetDateTime;
use ::time::format_description::well_known::Rfc3339;

#[derive(Debug)]
pub enum DateFormatError {
    /// Date parsing error
    TimeParsing(time::error::Parse),
    /// Integer to Month conversion error
    TimeComponentRange(time::error::ComponentRange),
    /// String to Integer conversion error
    IntParsing(std::num::ParseIntError),
    /// String splitting error
    StringSplit,
    /// Incorrectly formatted date error
    InvalidDate,
}

impl std::fmt::Display for DateFormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TimeParsing(err) => write!(f, "Couldn't parse date: {err}"),
            Self::TimeComponentRange(err) => write!(f, "Invalid month: {err}"),
            Self::IntParsing(err) => write!(f, "Couldn't parse string to integer: {err}"),
            Self::StringSplit => f.write_str("Couldn't split string"),
            Self::InvalidDate => f.write_str("Invalid date format"),
        }
    }
}

impl std::error::Error for DateFormatError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::TimeParsing(err) => Some(err),
            Self::TimeComponentRange(err) => Some(err),
            Self::IntParsing(err) => Some(err),
            _ => None,
        }
    }
}

impl From<time::error::Parse> for DateFormatError {
    fn from(value: time::error::Parse) -> Self {
        Self::TimeParsing(value)
    }
}

impl From<time::error::ComponentRange> for DateFormatError {
    fn from(value: time::error::ComponentRange) -> Self {
        Self::TimeComponentRange(value)
    }
}

impl From<std::num::ParseIntError> for DateFormatError {
    fn from(value: std::num::ParseIntError) -> Self {
        Self::IntParsing(value)
    }
}

/// FHIR instant type: <https://hl7.org/fhir/datatypes.html#instant>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Instant(#[serde(with = "time::serde::rfc3339")] pub OffsetDateTime);

/// FHIR date type: <https://hl7.org/fhir/datatypes.html#date>
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Date {
    /// Date in the format of YYYY
    Year(i32),
    /// Date in the format of YYYY-MM
    YearMonth(i32, time::Month),
    /// Date in the format of YYYY-MM-DD
    Date(time::Date),
}

/// FHIR dateTime type: <https://hl7.org/fhir/datatypes.html#dateTime>
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(untagged)]
pub enum DateTime {
    /// Date that does not contain time or timezone
    Date(Date),
    /// Full date and time
    DateTime(Instant),
}

/// FHIR time type: <https://hl7.org/fhir/datatypes.html#time>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Time(#[serde(with = "serde_time")] pub time::Time);

/// Serde module for serialize and deserialize function for the type.
mod serde_time {
    use serde::{Deserialize, Serialize};
    use time::format_description::FormatItem;
    use time::macros::format_description;

    /// Time format `hh:mm:ss`.
    const TIME_FORMAT: &[FormatItem<'_>] = format_description!("[hour]:[minute]:[second]");
    /// Time format for `hh:mm:ss[.SSS]`.
    const TIME_FORMAT_SUBSEC: &[FormatItem<'_>] = fhir_time_format();

    /// Time format with optional subseconds.
    const fn fhir_time_format() -> &'static [FormatItem<'static>] {
        /// Optional subseconds.
        const OPTIONAL_SUB_SECONDS: FormatItem<'_> =
            FormatItem::Optional(&FormatItem::Compound(format_description!(".[subsecond]")));
        &[FormatItem::Compound(TIME_FORMAT), OPTIONAL_SUB_SECONDS]
    }

    /// Serialize time, using subseconds iff appropriate.
    #[allow(clippy::trivially_copy_pass_by_ref, reason = "Parameter types are set by serde")]
    pub fn serialize<S>(time: &time::Time, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let format = if time.nanosecond() == 0 { TIME_FORMAT } else { TIME_FORMAT_SUBSEC };
        time.format(format).map_err(serde::ser::Error::custom)?.serialize(serializer)
    }

    /// Deserialize time, subseconds optional.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<time::Time, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        time::Time::parse(&string, TIME_FORMAT_SUBSEC).map_err(serde::de::Error::custom)
    }
}

impl Serialize for Date {
    /// Serialize date.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match &self {
            // Serialize YYYY
            Date::Year(year) => {
                if (1000..10000).contains(year) {
                    year.to_string().serialize(serializer)
                } else {
                    Err(serde::ser::Error::custom("Year is not 4 digits long"))
                }
            }
            // Serialize YYYY-MM
            Date::YearMonth(year, month) => {
                if (1000..10000).contains(year) {
                    serializer.serialize_str(&format!("{year}-{:02}", *month as u8))
                } else {
                    Err(serde::ser::Error::custom("Year is not 4 digits long"))
                }
            }
            // Serialize YYYY-MM-DD
            Date::Date(date) => {
                /// Full date format
                const FORMAT: &[time::format_description::FormatItem<'_>] =
                    time::macros::format_description!("[year]-[month]-[day]");
                date.format(FORMAT).map_err(serde::ser::Error::custom)?.serialize(serializer)
            }
        }
    }
}

impl FromStr for Date {
    type Err = DateFormatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Split date into parts
        // YYYY(1)-MM(2)-DD(3)
        match s.split('-').count() {
            1 => Ok(Date::Year(s.parse::<i32>()?)),
            2 => {
                let (year, month) = s.split_once('-').ok_or(DateFormatError::StringSplit)?;
                // Convert strings into integers
                let year = year.parse::<i32>()?;
                let month = month.parse::<u8>()?;

                Ok(Date::YearMonth(year, month.try_into()?))
            }
            3 => {
                /// Full date format
                const FORMAT: &[time::format_description::FormatItem<'_>] =
                    time::macros::format_description!("[year]-[month]-[day]");
                Ok(Date::Date(time::Date::parse(s, FORMAT)?))
            }
            _ => Err(DateFormatError::InvalidDate),
        }
    }
}

impl<'de> Deserialize<'de> for Date {
    /// Deserialize date.
    fn deserialize<D>(deserializer: D) -> Result<Date, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        Date::from_str(&string).map_err(serde::de::Error::custom)
    }
}

impl FromStr for DateTime {
    type Err = DateFormatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains('T') {
            let instant = Instant::from_str(s)?;
            Ok(DateTime::DateTime(instant))
        } else {
            let date = Date::from_str(s)?;
            Ok(DateTime::Date(date))
        }
    }
}

impl<'de> Deserialize<'de> for DateTime {
    /// Deserialize datetime.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        Self::from_str(&string).map_err(serde::de::Error::custom)
    }
}

impl FromStr for Instant {
    type Err = time::error::Parse;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Instant(OffsetDateTime::parse(s, &Rfc3339)?))
    }
}

impl std::fmt::Display for Instant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.format(&Rfc3339).map_err(|_| std::fmt::Error)?)
    }
}

impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Date::Year(year) => write!(f, "{year}"),
            Date::YearMonth(year, month) => write!(f, "{year}-{:02}", *month as u8),
            Date::Date(date) => write!(f, "{:04}-{:02}-{:02}", date.year(), date.month() as u8, date.day()),
        }
    }
}

impl std::fmt::Display for DateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DateTime::Date(date) => write!(f, "{date}"),
            DateTime::DateTime(instant) => write!(f, "{instant}"),
        }
    }
}

impl PartialOrd for Date {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Date::Date(ld), r) => ld.partial_cmp(r),
            (l, Date::Date(rd)) => l.partial_cmp(rd),
            (Date::Year(ly), Date::Year(ry)) => ly.partial_cmp(ry),
            (Date::Year(ly), Date::YearMonth(ry, _rm)) => ly.partial_cmp(ry),
            (Date::YearMonth(ly, _lm), Date::Year(ry)) => ly.partial_cmp(ry),
            (Date::YearMonth(ly, lm), Date::YearMonth(ry, rm)) => match ly.partial_cmp(ry)? {
                Ordering::Less => Some(Ordering::Less),
                Ordering::Greater => Some(Ordering::Greater),
                Ordering::Equal => (*lm as u8).partial_cmp(&(*rm as u8)),
            },
        }
    }
}

impl PartialOrd for DateTime {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (DateTime::Date(ld), DateTime::Date(rd)) => ld.partial_cmp(rd),
            (DateTime::Date(ld), DateTime::DateTime(Instant(rdtm))) => ld.partial_cmp(&rdtm.date()),
            (DateTime::DateTime(Instant(ldtm)), DateTime::Date(rd)) => ldtm.date().partial_cmp(rd),
            (DateTime::DateTime(ldtm), DateTime::DateTime(rdtm)) => ldtm.partial_cmp(rdtm),
        }
    }
}

impl PartialEq<time::Date> for Date {
    fn eq(&self, other: &time::Date) -> bool {
        match self {
            Self::Year(year) => *year == other.year(),
            Self::YearMonth(year, month) => *year == other.year() && *month == other.month(),
            Self::Date(date) => date == other,
        }
    }
}

impl PartialEq<Date> for time::Date {
    fn eq(&self, other: &Date) -> bool {
        match other {
            Date::Year(year) => self.year() == *year,
            Date::YearMonth(year, month) => self.year() == *year && self.month() == *month,
            Date::Date(date) => self == date,
        }
    }
}

impl PartialOrd<time::Date> for Date {
    fn partial_cmp(&self, other: &time::Date) -> Option<Ordering> {
        match self {
            Date::Year(year) => year.partial_cmp(&other.year()),
            Date::YearMonth(year, month) => match year.partial_cmp(&other.year())? {
                Ordering::Less => Some(Ordering::Less),
                Ordering::Greater => Some(Ordering::Greater),
                Ordering::Equal => (*month as u8).partial_cmp(&(other.month() as u8)),
            },
            Date::Date(date) => date.partial_cmp(other),
        }
    }
}

impl PartialOrd<Date> for time::Date {
    fn partial_cmp(&self, other: &Date) -> Option<Ordering> {
        match other {
            Date::Year(year) => self.year().partial_cmp(year),
            Date::YearMonth(year, month) => match self.year().partial_cmp(year)? {
                Ordering::Less => Some(Ordering::Less),
                Ordering::Greater => Some(Ordering::Greater),
                Ordering::Equal => (self.month() as u8).partial_cmp(&(*month as u8)),
            },
            Date::Date(date) => self.partial_cmp(date),
        }
    }
}

impl PartialEq<OffsetDateTime> for DateTime {
    fn eq(&self, other: &OffsetDateTime) -> bool {
        match self {
            Self::Date(date) => *date == other.date(),
            Self::DateTime(Instant(datetime)) => datetime == other,
        }
    }
}

impl PartialEq<DateTime> for OffsetDateTime {
    fn eq(&self, other: &DateTime) -> bool {
        match other {
            DateTime::Date(date) => self.date() == *date,
            DateTime::DateTime(Instant(datetime)) => self == datetime,
        }
    }
}

impl PartialOrd<OffsetDateTime> for DateTime {
    fn partial_cmp(&self, other: &OffsetDateTime) -> Option<Ordering> {
        match self {
            DateTime::Date(date) => date.partial_cmp(&other.date()),
            DateTime::DateTime(Instant(datetime)) => datetime.partial_cmp(other),
        }
    }
}

impl PartialOrd<DateTime> for OffsetDateTime {
    fn partial_cmp(&self, other: &DateTime) -> Option<Ordering> {
        match other {
            DateTime::Date(date) => self.date().partial_cmp(date),
            DateTime::DateTime(Instant(datetime)) => self.partial_cmp(datetime),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(Date::Year(2024), "2024")]
    #[case(Date::YearMonth(2024, time::Month::March), "2024-03")]
    #[case(Date::YearMonth(2024, time::Month::December), "2024-12")]
    #[case(Date::Date(time::macros::date!(2024 - 03 - 07)), "2024-03-07")]
    fn displays_date(#[case] date: Date, #[case] expected: &str) {
        assert_eq!(date.to_string(), expected);
    }

    #[rstest]
    #[case(time::macros::datetime!(2024 - 03 - 07 12:34:56 UTC), "2024-03-07T12:34:56Z")]
    #[case(time::macros::datetime!(2024 - 03 - 07 12:34:56.789 UTC), "2024-03-07T12:34:56.789Z")]
    #[case(time::macros::datetime!(2024 - 03 - 07 12:34:56 +2), "2024-03-07T12:34:56+02:00")]
    fn displays_instant(#[case] datetime: OffsetDateTime, #[case] expected: &str) {
        assert_eq!(Instant(datetime).to_string(), expected);
    }

    #[rstest]
    #[case("2024")]
    #[case("2024-03")]
    #[case("2024-03-07")]
    #[case("2024-03-07T12:34:56Z")]
    #[case("2024-03-07T12:34:56.789Z")]
    #[case("2024-03-07T12:34:56+02:00")]
    fn datetime_round_trip(#[case] input: &str) {
        assert_eq!(DateTime::from_str(input).unwrap().to_string(), input);
    }

    #[rstest]
    #[case(DateTime::Date(Date::Year(2024)), "2024")]
    #[case(DateTime::Date(Date::YearMonth(2024, time::Month::March)), "2024-03")]
    #[case(DateTime::Date(Date::Date(time::macros::date!(2024 - 03 - 07))), "2024-03-07")]
    #[case(
        DateTime::DateTime(Instant(time::macros::datetime!(2024 - 03 - 07 12:34:56 UTC))),
        "2024-03-07T12:34:56Z"
    )]
    fn displays_datetime(#[case] datetime: DateTime, #[case] expected: &str) {
        assert_eq!(datetime.to_string(), expected);
    }
}
