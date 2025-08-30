use json::{Json, Pair};
use std::{fmt::Debug, str::FromStr};
use timing_macro::time_function;

enum JsonError {
    Pair(String),
    Key(String),
    Value(String),
}

impl Debug for JsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pair(pair) => f.debug_struct("JsonError").field(" Pair", &pair).finish(),
            Self::Key(key) => f.debug_struct("JsonError").field("Key", &key).finish(),
            Self::Value(value) => f.debug_struct("JsonError").field("Value", &value).finish(),
        }
    }
}

// wrappers to get around the orphan rule
struct JsonWrapper(Json);
struct PairWrapper(Pair);

impl FromStr for JsonWrapper {
    type Err = JsonError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut pairs = Vec::new();

        for object in s
            .split_once(':')
            .unwrap()
            .1
            .split_once('[')
            .unwrap()
            .1
            .split('{')
            .skip(1)
        {
            match object.parse::<PairWrapper>() {
                Ok(p) => pairs.push(p.0),

                Err(e) => return Err(e),
            }
        }

        Ok(JsonWrapper(Json { pairs }))
    }
}

fn parse_f64(s: &str) -> Result<f64, JsonError> {
    if s.contains('}') {
        return parse_f64(s.split_once('}').unwrap().0);
    }
    match s.parse::<f64>() {
        Ok(v) => Ok(v),
        Err(_) => Err(JsonError::Value(s.to_string())),
    }
}

impl FromStr for PairWrapper {
    type Err = JsonError;

    // Assumes we're inside the opening bracket
    // s: r#""x0": 123.215645, "y0": 1.6546, "x1": -15.5466, "y1": -56.56464}"
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut pair = Pair {
            x0: 0.0,
            y0: 0.0,
            x1: 0.0,
            y1: 0.0,
        };

        for kv in s.split(',') {
            if kv.is_empty() {
                continue;
            };
            match kv.split_once(':') {
                Some((key, value)) => match key {
                    "\"x0\"" => pair.x0 = parse_f64(value)?,
                    "\"x1\"" => pair.x1 = parse_f64(value)?,
                    "\"y0\"" => pair.y0 = parse_f64(value)?,
                    "\"y1\"" => pair.y1 = parse_f64(value)?,
                    _ => return Err(JsonError::Key(key.to_string())),
                },
                None => return Err(JsonError::Pair(kv.to_string())),
            }
        }

        Ok(PairWrapper(pair))
    }
}

/// ! __Not a general use json parser__ !
///
/// Parses the json::Json object from a string.
///
/// Assumes string is a json string formatted as:
/// ```json
/// {
///     "pairs": [
///         {
///             "x0": <f64>,
///             "y0": <f64>,
///             "x1": <f64>,
///             "y1": <f64>
///         },
///         ...
///     ]
/// }
/// ```
#[time_function]
pub fn parse(input: &str) -> Json {
    input.parse::<JsonWrapper>().expect("error parsing json").0
}
