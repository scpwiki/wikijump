use crate::parse::RelationSettings;
use crate::types::GenerateMethod;
use syn::{Type, TypePath};

/// Validate that `RelationSettings` are being parsed as expected.
#[test]
fn parse() {
    fn parse_settings(input: &str) -> RelationSettings {
        syn::parse_str(input).expect("failed to parse macro code")
    }

    // Test cases

    {
        let settings = parse_settings(
            "
            name => UserBlock,
            dest => blocked_user: User,
            from => blocking_user: User,
            data => UserBlockData,
            ",
        );
        assert_eq!(settings.relation_name.to_string(), "UserBlock");
        assert_eq!(settings.field_name.to_string(), "user_block");
        assert_eq!(settings.dest.0.to_string(), "blocked_user");
        assert!(matches!(settings.dest.1, Type::Path(TypePath { .. })));
        assert_eq!(settings.from.0.to_string(), "blocking_user");
        assert!(matches!(settings.from.1, Type::Path(TypePath { .. })));
        assert_eq!(settings.create_fn, GenerateMethod::ImplPublic);
        assert_eq!(settings.remove_fn, GenerateMethod::ImplPublic);
    }
}

/// Validate that code generation is as expected given some `RelationSettings`.
#[test]
fn generate() {
    // TODO
}
