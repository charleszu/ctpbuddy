use std::{env, fs, path::PathBuf};

use serde_json::Value;

fn string(value: &Value, context: &str) -> String {
    value
        .as_str()
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| panic!("shared refdata contract {context} must be a string"))
}

fn strings(value: &Value, context: &str) -> Vec<String> {
    let values = value
        .as_array()
        .unwrap_or_else(|| panic!("shared refdata contract {context} must be an array"));
    let out = values
        .iter()
        .map(|item| string(item, context))
        .collect::<Vec<_>>();
    let mut unique = out.clone();
    unique.sort();
    unique.dedup();
    if unique.len() != out.len() {
        panic!("shared refdata contract {context} contains duplicate values");
    }
    out
}

fn rust_strings(values: &[String]) -> String {
    values
        .iter()
        .map(|value| format!("{value:?}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn main() {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let path = manifest_dir.join("../../py/ctpbuddy/contracts/refdata_contract.json");
    let text = fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "cannot read shared refdata contract {}: {error}",
            path.display()
        )
    });
    let root: Value = serde_json::from_str(&text).unwrap_or_else(|error| {
        panic!(
            "shared refdata contract {} is invalid JSON: {error}",
            path.display()
        )
    });
    if string(root.get("schema").unwrap_or(&Value::Null), "schema")
        != "ctpbuddy.refdata-contract/v1"
        || string(root.get("contract").unwrap_or(&Value::Null), "contract") != "ctpbuddy.refdata"
    {
        panic!("shared refdata contract schema/contract mismatch");
    }
    let contract_version = root
        .get("contract_version")
        .and_then(Value::as_i64)
        .filter(|value| *value >= 0 && *value <= i32::MAX as i64)
        .unwrap_or_else(|| panic!("shared refdata contract_version must be an integer"));
    let format_version = root
        .get("format_version")
        .and_then(Value::as_i64)
        .filter(|value| *value >= 0 && *value <= i32::MAX as i64)
        .unwrap_or_else(|| panic!("shared refdata format_version must be an integer"));

    let tables = root
        .get("tables")
        .and_then(Value::as_object)
        .unwrap_or_else(|| panic!("shared refdata contract tables must be an object"));
    let table_names = [
        ("instruments", "INSTRUMENT"),
        ("margin_rates", "MARGIN_RATE"),
        ("commission_rates", "COMMISSION_RATE"),
        ("order_comm_rates", "ORDER_COMM_RATE"),
        ("trading_params", "TRADING_PARAM"),
        ("cross_margin_rules", "CROSS_MARGIN_RULE"),
    ];
    if tables.len() != table_names.len()
        || table_names
            .iter()
            .any(|(name, _)| !tables.contains_key(*name))
    {
        panic!("shared refdata contract tables are incomplete");
    }

    let mut generated = String::new();
    let mut all_fields = Vec::new();
    generated.push_str(&format!(
        "pub const CONTRACT: &str = {:?};\npub const CONTRACT_VERSION: i32 = {contract_version};\npub const FORMAT_VERSION: i32 = {format_version};\n",
        "ctpbuddy.refdata"
    ));
    for (name, ident) in table_names {
        let table =
            tables.get(name).unwrap().as_object().unwrap_or_else(|| {
                panic!("shared refdata contract table {name} must be an object")
            });
        let file = string(
            table.get("file").unwrap_or(&Value::Null),
            &format!("{name}.file"),
        );
        if table.get("required").and_then(Value::as_bool).is_none() {
            panic!("shared refdata contract {name}.required must be a boolean");
        }
        let fields = strings(
            table.get("fields").unwrap_or(&Value::Null),
            &format!("{name}.fields"),
        );
        let required_fields = strings(
            table.get("required_fields").unwrap_or(&Value::Null),
            &format!("{name}.required_fields"),
        );
        if required_fields.iter().any(|field| !fields.contains(field)) {
            panic!("shared refdata contract {name}.required_fields contains an unknown field");
        }
        all_fields.extend(fields.iter().cloned());
        generated.push_str(&format!(
            "pub const {ident}_FILE: &str = {file:?};\npub const {ident}_FIELDS: &[&str] = &[{}];\npub const {ident}_REQUIRED_FIELDS: &[&str] = &[{}];\n",
            rust_strings(&fields),
            rust_strings(&required_fields),
        ));
    }
    let numeric_fields = strings(
        root.get("numeric_fields").unwrap_or(&Value::Null),
        "numeric_fields",
    );
    if numeric_fields
        .iter()
        .any(|field| !all_fields.contains(field))
    {
        panic!("shared refdata contract numeric_fields contains an unknown field");
    }
    generated.push_str(&format!(
        "pub const NUMERIC_FIELDS: &[&str] = &[{}];\n",
        rust_strings(&numeric_fields)
    ));
    let defaults = root
        .get("defaults")
        .and_then(Value::as_object)
        .unwrap_or_else(|| panic!("shared refdata contract defaults must be an object"));
    for (key, ident) in [
        ("margin_price_type", "DEFAULT_MARGIN_PRICE_TYPE"),
        ("broker_id", "DEFAULT_BROKER_ID"),
        ("investor_id", "DEFAULT_INVESTOR_ID"),
        ("hedge_flag", "DEFAULT_HEDGE_FLAG"),
    ] {
        generated.push_str(&format!(
            "pub const {ident}: &str = {:?};\n",
            string(
                defaults.get(key).unwrap_or(&Value::Null),
                &format!("defaults.{key}")
            )
        ));
    }
    let enums = root
        .get("enums")
        .and_then(Value::as_object)
        .and_then(|items| items.get("margin_price_type"))
        .and_then(Value::as_object)
        .unwrap_or_else(|| {
            panic!("shared refdata contract enums.margin_price_type must be an object")
        });
    let mut enum_values = enums
        .iter()
        .map(|(key, value)| (key.clone(), string(value, "enums.margin_price_type value")))
        .collect::<Vec<_>>();
    enum_values.sort_by(|left, right| left.0.cmp(&right.0));
    generated.push_str("pub const MARGIN_PRICE_TYPES: &[(&str, &str)] = &[");
    for (index, (key, value)) in enum_values.iter().enumerate() {
        if index > 0 {
            generated.push_str(", ");
        }
        generated.push_str(&format!("({key:?}, {value:?})"));
    }
    generated.push_str("];\n");

    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("refdata_contract.rs");
    fs::write(&out, generated).unwrap_or_else(|error| {
        panic!(
            "cannot write generated refdata contract {}: {error}",
            out.display()
        )
    });
    println!("cargo:rerun-if-changed={}", path.display());
}
