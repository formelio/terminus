use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize};

#[derive(
    serde::Deserialize,
    serde::Serialize,
    strum::AsRefStr,
    strum::EnumString,
    strum::Display,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    PartialEq,
)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum SearchComparator {
    Ap,
    Eb,
    Eq,
    Ge,
    Gt,
    Le,
    Lt,
    Ne,
    Sa,
}

#[derive(Debug)]
pub struct ParameterWithComparator<P> {
    pub comparator: Option<SearchComparator>,
    pub parameter: P,
}

impl<P: std::fmt::Display> std::fmt::Display for ParameterWithComparator<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(comparator) = self.comparator {
            f.write_str(comparator.as_ref())?;
        }
        self.parameter.fmt(f)
    }
}

impl<P: std::fmt::Display> Serialize for ParameterWithComparator<P> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de, P> Deserialize<'de> for ParameterWithComparator<P>
where
    P: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;

        let (comparator, rest) = match raw.get(..2).and_then(|p| SearchComparator::from_str(p).ok()) {
            Some(comparator) => (Some(comparator), &raw[2..]),
            None => (None, raw.as_str()),
        };

        let parameter = P::deserialize(serde::de::value::StrDeserializer::new(rest))?;

        Ok(Self { comparator, parameter })
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Or<P>(pub Vec<P>);

impl<P: std::fmt::Display> Serialize for Or<P> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.iter().map(ToString::to_string).collect::<Vec<_>>().join(",").serialize(serializer)
    }
}

impl<'de, P> Deserialize<'de> for Or<P>
where
    P: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;

        raw.split(',')
            .filter(|v| !v.is_empty())
            .map(|v| P::deserialize(serde::de::value::StrDeserializer::new(v)))
            .collect::<Result<Vec<P>, _>>()
            .map(Or)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::date_time::{Date, DateTime};

    #[rstest]
    #[case("\"ge2024\"", Some(SearchComparator::Ge), "2024")]
    #[case("\"2024\"", None, "2024")]
    fn parses_string_comparator(
        #[case] input: &str,
        #[case] comparator: Option<SearchComparator>,
        #[case] parameter: &str,
    ) {
        let parsed: ParameterWithComparator<String> = serde_json::from_str(input).unwrap();
        assert_eq!(parsed.comparator, comparator);
        assert_eq!(parsed.parameter, parameter);
    }

    #[rstest]
    #[case("\"ge2024\"", Some(SearchComparator::Ge), DateTime::Date(Date::Year(2024)))]
    #[case("\"2024\"", None, DateTime::Date(Date::Year(2024)))]
    fn parses_date_comparator(
        #[case] input: &str,
        #[case] comparator: Option<SearchComparator>,
        #[case] parameter: DateTime,
    ) {
        let parsed: ParameterWithComparator<DateTime> = serde_json::from_str(input).unwrap();
        assert_eq!(parsed.comparator, comparator);
        assert_eq!(parsed.parameter, parameter);
    }

    #[rstest]
    #[case("\"zz2024\"")]
    fn rejects_invalid_date_comparator(#[case] input: &str) {
        serde_json::from_str::<ParameterWithComparator<DateTime>>(input).unwrap_err();
    }

    #[rstest]
    #[case("\"a,b,c\"", vec!["a", "b", "c"])]
    #[case("\"\"", vec![])]
    fn or_round_trip(#[case] input: &str, #[case] expected: Vec<&str>) {
        let parsed: Or<String> = serde_json::from_str(input).unwrap();
        assert_eq!(parsed.0, expected);
        assert_eq!(serde_json::to_string(&parsed).unwrap(), input);
    }
}
