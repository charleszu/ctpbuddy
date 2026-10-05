//! JSON for admin/AUTH payloads and the journal. Parsing and string escaping
//! are delegated to `serde_json` (RFC 8259 compliant, recursion-limited);
//! this module only keeps the small `Value` facade the rest of the server uses,
//! with insertion-ordered objects and `NaN`/`Infinity` serialized as `null`.

use serde::de::{Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use std::collections::BTreeMap;
use std::fmt::{self, Write as _};

#[derive(Clone, Debug)]
pub enum Value {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Value>),
    Obj(Vec<(String, Value)>),
}

impl Value {
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Obj(pairs) => pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn get_str(&self, key: &str) -> Option<String> {
        self.get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    pub fn get_num(&self, key: &str) -> Option<f64> {
        self.get(key).and_then(|v| match v {
            Value::Num(n) => Some(*n),
            Value::Str(s) => s.trim().parse().ok(),
            _ => None,
        })
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key).and_then(|v| match v {
            Value::Bool(b) => Some(*b),
            Value::Str(s) => match s.as_str() {
                "true" | "1" => Some(true),
                "false" | "0" => Some(false),
                _ => None,
            },
            _ => None,
        })
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_obj(&self) -> Option<&[(String, Value)]> {
        match self {
            Value::Obj(pairs) => Some(pairs),
            _ => None,
        }
    }

    pub fn as_num(&self) -> Option<f64> {
        match self {
            Value::Num(n) => Some(*n),
            _ => None,
        }
    }

    pub fn to_json(&self) -> String {
        let mut s = String::new();
        self.write(&mut s);
        s
    }

    fn write(&self, out: &mut String) {
        match self {
            Value::Null => out.push_str("null"),
            Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            Value::Num(n) => {
                if !n.is_finite() {
                    // JSON has no NaN/Infinity literal; emit null rather than
                    // corrupt the journal / admin payload.
                    out.push_str("null");
                } else if n.fract() == 0.0 && n.abs() < 1e15 {
                    let _ = write!(out, "{}", *n as i64);
                } else {
                    let _ = write!(out, "{n}");
                }
            }
            Value::Str(s) => write_json_str(out, s),
            Value::Arr(items) => {
                out.push('[');
                for (i, v) in items.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    v.write(out);
                }
                out.push(']');
            }
            Value::Obj(pairs) => {
                out.push('{');
                for (i, (k, v)) in pairs.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    write_json_str(out, k);
                    out.push(':');
                    v.write(out);
                }
                out.push('}');
            }
        }
    }
}

fn write_json_str(out: &mut String, s: &str) {
    // serde_json escapes quotes, backslashes and control characters.
    out.push_str(&serde_json::to_string(s).expect("string serialization is infallible"));
}

// ---- builder helpers ----
pub fn s(x: &str) -> Value {
    Value::Str(x.to_string())
}
pub fn n(x: f64) -> Value {
    Value::Num(x)
}
pub fn b(x: bool) -> Value {
    Value::Bool(x)
}

/// Parse JSON text. Returns a human-readable error string on failure.
pub fn parse(text: &str) -> Result<Value, String> {
    serde_json::from_str::<Value>(text).map_err(|e| e.to_string())
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Value, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = Value;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("any JSON value")
            }
            fn visit_unit<E>(self) -> Result<Value, E> {
                Ok(Value::Null)
            }
            fn visit_none<E>(self) -> Result<Value, E> {
                Ok(Value::Null)
            }
            fn visit_bool<E>(self, b: bool) -> Result<Value, E> {
                Ok(Value::Bool(b))
            }
            fn visit_i64<E>(self, n: i64) -> Result<Value, E> {
                Ok(Value::Num(n as f64))
            }
            fn visit_u64<E>(self, n: u64) -> Result<Value, E> {
                Ok(Value::Num(n as f64))
            }
            fn visit_f64<E>(self, n: f64) -> Result<Value, E> {
                Ok(Value::Num(n))
            }
            fn visit_str<E>(self, s: &str) -> Result<Value, E> {
                Ok(Value::Str(s.to_string()))
            }
            fn visit_string<E>(self, s: String) -> Result<Value, E> {
                Ok(Value::Str(s))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Value, A::Error> {
                let mut items = Vec::new();
                while let Some(v) = seq.next_element()? {
                    items.push(v);
                }
                Ok(Value::Arr(items))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
                let mut pairs = Vec::new();
                while let Some((k, v)) = map.next_entry::<String, Value>()? {
                    pairs.push((k, v));
                }
                Ok(Value::Obj(pairs))
            }
        }
        d.deserialize_any(V)
    }
}

/// Sorted-key object for stable status output.
pub fn obj_sorted(pairs: Vec<(String, Value)>) -> Value {
    let map: BTreeMap<String, Value> = pairs.into_iter().collect();
    Value::Obj(map.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_finite_numbers_serialize_as_null() {
        let v = obj_sorted(vec![
            ("nan".into(), n(f64::NAN)),
            ("inf".into(), n(f64::INFINITY)),
            ("neg_inf".into(), n(f64::NEG_INFINITY)),
            ("one".into(), n(1.0)),
            ("half".into(), n(0.5)),
        ]);
        let text = v.to_json();
        assert_eq!(
            text,
            r#"{"half":0.5,"inf":null,"nan":null,"neg_inf":null,"one":1}"#
        );
        // the output must round-trip through our own parser (valid JSON)
        let back = parse(&text).unwrap();
        assert!(matches!(back.get("nan"), Some(Value::Null)));
        assert_eq!(back.get_num("one"), Some(1.0));
    }

    #[test]
    fn parses_nested_unicode_and_escapes() {
        let v = parse(r#"{"a":{"b":[1,2.5,"x\u00e9\n"]},"s":"\ud83d\ude00","k":1e3}"#).unwrap();
        assert_eq!(v.get_num("k"), Some(1000.0));
        assert_eq!(v.get_str("s").as_deref(), Some("\u{1F600}"));
        let round = parse(&v.to_json()).unwrap();
        assert_eq!(round.to_json(), v.to_json());
    }

    #[test]
    fn rejects_malformed_and_deeply_nested_input() {
        for bad in [
            "",
            "{",
            "{\"a\":}",
            "[1,]",
            "01",
            "{\"a\":1} x",
            "nul",
            "\"\\q\"",
        ] {
            assert!(parse(bad).is_err(), "should reject {bad:?}");
        }
        let deep = "[".repeat(10_000);
        assert!(parse(&deep).is_err());
    }

    #[test]
    fn preserves_object_key_order() {
        let v = parse(r#"{"z":1,"a":2,"m":3}"#).unwrap();
        let keys: Vec<&str> = v
            .as_obj()
            .unwrap()
            .iter()
            .map(|(k, _)| k.as_str())
            .collect();
        assert_eq!(keys, ["z", "a", "m"]);
    }
}
