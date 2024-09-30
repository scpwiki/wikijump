/*
 * web/provided_value.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2024 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

/// Denotes that a field is optional in a struct.
///
/// This is meant to be used when doing `UPDATE` operations,
/// since excluding the field entirely is different from setting
/// it to null (`None`).
///
/// The `Unset` variant can only be constructed if the field is absent.
#[derive(Serialize, Deserialize, Debug, Default, Clone, Hash, PartialEq, Eq)]
#[serde(untagged)]
pub enum ProvidedValue<T> {
    Set(T),

    #[serde(skip)]
    #[default]
    Unset,
}

impl<T> ProvidedValue<T> {
    pub fn to_option(&self) -> Option<&T> {
        match self {
            ProvidedValue::Set(ref value) => Some(value),
            ProvidedValue::Unset => None,
        }
    }
}

impl<T> ProvidedValue<T>
where
    T: Into<sea_orm::Value>,
{
    pub fn into_active_value(self) -> sea_orm::ActiveValue<T> {
        match self {
            ProvidedValue::Set(value) => sea_orm::ActiveValue::Set(value),
            ProvidedValue::Unset => sea_orm::ActiveValue::NotSet,
        }
    }
}

impl<T> From<ProvidedValue<T>> for Option<T> {
    fn from(value: ProvidedValue<T>) -> Option<T> {
        match value {
            ProvidedValue::Set(value) => Some(value),
            ProvidedValue::Unset => None,
        }
    }
}

#[test]
fn serde() {
    use serde_json::json;

    #[derive(Serialize, Deserialize, Debug)]
    struct Object {
        #[serde(default)]
        field: ProvidedValue<Option<String>>,
    }

    // Deserialization

    macro_rules! check_deser {
        ($value:expr, $expected:expr $(,)?) => {{
            let object: Object =
                serde_json::from_value($value).expect("Unable to deserialize JSON");

            assert_eq!(
                object.field, $expected,
                "Actual optional item doesn't match expected",
            );
        }};
    }

    check_deser!(json!({}), ProvidedValue::Unset);
    check_deser!(json!({ "field": null }), ProvidedValue::Set(None));
    check_deser!(
        json!({"field": "apple"}),
        ProvidedValue::Set(Some(str!("apple"))),
    );

    // Serialization

    macro_rules! check_ser {
        ($field:expr, $expected:expr $(,)?) => {{
            let object = Object { field: $field };
            let json = serde_json::to_string(&object).expect("Unable to serialize JSON");

            assert_eq!(
                json, $expected,
                "Actual generated JSON doesn't match expected",
            );
        }};
    }

    check_ser!(ProvidedValue::Unset, "{}");
    check_ser!(ProvidedValue::Set(None), r#"{"field":null}"#);
    check_ser!(
        ProvidedValue::Set(Some(str!("banana"))),
        r#"{"field":"banana"}"#,
    );
}
