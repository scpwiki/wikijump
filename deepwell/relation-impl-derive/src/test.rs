use crate::parse::RelationSettings;
use crate::types::GenerateMethod;
use syn::Type;

#[test]
fn parse() {
    fn parse_settings(input: &str) -> RelationSettings {
        syn::parse_str(input).expect("failed to parse macro code")
    }

    // Test cases

    {
        let settings = parse_settings(
            "
            name => Foo,
            dest => first: User,
            from => second: User,
            define_struct => false
            ",
        );
        assert_eq!(settings.relation_name.to_string(), "Foo");
        assert_eq!(settings.field_name.to_string(), "foo");
        assert_eq!(settings.dest.0.to_string(), "first");
        assert!(matches!(settings.dest.1, Type::Path(_)));
        assert_eq!(settings.from.0.to_string(), "second");
        assert!(matches!(settings.from.1, Type::Path(_)));
        assert!(settings.data_type.is_none());
        assert_eq!(settings.create_fn, GenerateMethod::ImplPublic);
        assert_eq!(settings.remove_fn, GenerateMethod::ImplPublic);
        assert!(!settings.define_struct);
    }

    {
        let settings = parse_settings(
            "
            name => PageStar,
            dest => page_id: Page,
            from => user_id: User,
            data => (),
            ",
        );
        assert_eq!(settings.relation_name.to_string(), "PageStar");
        assert_eq!(settings.field_name.to_string(), "page_star");
        assert_eq!(settings.dest.0.to_string(), "page_id");
        assert!(matches!(settings.dest.1, Type::Path(_)));
        assert_eq!(settings.from.0.to_string(), "user_id");
        assert!(matches!(settings.from.1, Type::Path(_)));
        assert!(settings.data_type.is_none());
        assert_eq!(settings.create_fn, GenerateMethod::ImplPublic);
        assert_eq!(settings.remove_fn, GenerateMethod::ImplPublic);
        assert!(settings.define_struct);
    }

    {
        let settings = parse_settings(
            "
            name => PageWatch,
            dest => page_id: Page,
            from => user_id: User,
            ",
        );
        assert_eq!(settings.relation_name.to_string(), "PageWatch");
        assert_eq!(settings.field_name.to_string(), "page_watch");
        assert_eq!(settings.dest.0.to_string(), "page_id");
        assert!(matches!(settings.dest.1, Type::Path(_)));
        assert_eq!(settings.from.0.to_string(), "user_id");
        assert!(matches!(settings.from.1, Type::Path(_)));
        assert!(settings.data_type.is_none());
        assert_eq!(settings.create_fn, GenerateMethod::ImplPublic);
        assert_eq!(settings.remove_fn, GenerateMethod::ImplPublic);
        assert!(settings.define_struct);
    }

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
        assert!(matches!(settings.dest.1, Type::Path(_)));
        assert_eq!(settings.from.0.to_string(), "blocking_user");
        assert!(matches!(settings.from.1, Type::Path(_)));
        assert!(matches!(settings.data_type, Some(Type::Path(_))));
        assert_eq!(settings.create_fn, GenerateMethod::ImplPublic);
        assert_eq!(settings.remove_fn, GenerateMethod::ImplPublic);
        assert!(settings.define_struct);
    }

    {
        let settings = parse_settings(
            "
            name => SiteBan,
            dest => site_id: Site,
            from => user_id: User,
            data => SiteBanData,
            create_fn => false,
            ",
        );
        assert_eq!(settings.relation_name.to_string(), "SiteBan");
        assert_eq!(settings.field_name.to_string(), "site_ban");
        assert_eq!(settings.dest.0.to_string(), "site_id");
        assert!(matches!(settings.dest.1, Type::Path(_)));
        assert_eq!(settings.from.0.to_string(), "user_id");
        assert!(matches!(settings.from.1, Type::Path(_)));
        assert!(matches!(settings.data_type, Some(Type::Path(_))));
        assert_eq!(settings.create_fn, GenerateMethod::NoImpl);
        assert_eq!(settings.remove_fn, GenerateMethod::ImplPublic);
        assert!(settings.define_struct);
    }

    {
        let settings = parse_settings(
            "
            name => UserBotOwner,
            dest => bot_user: User,
            from => owner_user: User,
            data => UserBotMetadata,
            create_fn => fn,
            define_struct => false,
            ",
        );
        assert_eq!(settings.relation_name.to_string(), "UserBotOwner");
        assert_eq!(settings.field_name.to_string(), "user_bot_owner");
        assert_eq!(settings.dest.0.to_string(), "bot_user");
        assert!(matches!(settings.dest.1, Type::Path(_)));
        assert_eq!(settings.from.0.to_string(), "owner_user");
        assert!(matches!(settings.from.1, Type::Path(_)));
        assert!(matches!(settings.data_type, Some(Type::Path(_))));
        assert_eq!(settings.create_fn, GenerateMethod::ImplPrivate);
        assert_eq!(settings.remove_fn, GenerateMethod::ImplPublic);
        assert!(!settings.define_struct);
    }
}
