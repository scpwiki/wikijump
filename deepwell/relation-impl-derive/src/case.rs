/// Convert `PascalCase` to `snake_case`.
pub fn pascal_to_snake_case(value: &str) -> String {
    let mut output = String::new();
    for (i, ch) in value.char_indices() {
        if i > 0 && ch.is_uppercase() {
            output.push('_');
        }
        output.push(ch.to_ascii_lowercase());
    }
    output
}

#[test]
fn test_pascal_to_snake_case() {
    macro_rules! test {
        ($pascal_case:expr, $snake_case:expr $(,)?) => {{
            let actual_snake_case = pascal_to_snake_case($pascal_case);
            let expected_snake_case = $snake_case;
            assert_eq!(
                actual_snake_case, expected_snake_case,
                "actual snake_case conversion doesn't match expected",
            );
        }};
    }

    test!("", "");
    test!("Foo", "foo");
    test!("FooBar", "foo_bar");
    test!(
        "AVeryLongNameWithManyParts",
        "a_very_long_name_with_many_parts",
    );
}
