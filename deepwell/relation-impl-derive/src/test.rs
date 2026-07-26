use crate::parse::RelationSettings;
use syn::Type;

#[test]
fn parse() {
    fn parse_settings(input: &str) -> RelationSettings {
        syn::parse_str(input).expect("failed to parse macro code")
    }

    fn assert_type(t_type: Type, name: &str) {
        let Type::Path(path_type) = t_type else {
            panic!("type does not match path with '{name}'")
        };

        assert!(path_type.attrs.is_empty(), "attributes in type");
        assert!(path_type.qself.is_none(), "QSelf in type is set");
        assert!(
            path_type.path.is_ident(name),
            "type path doesn't match expected",
        );
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
        assert_type(settings.dest.1, "User");
        assert_eq!(settings.from.0.to_string(), "second");
        assert_type(settings.from.1, "User");
        assert!(settings.data_type.is_none());
        assert!(settings.create_fn);
        assert!(settings.remove_fn);
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
        assert_type(settings.dest.1, "Page");
        assert_eq!(settings.from.0.to_string(), "user_id");
        assert_type(settings.from.1, "User");
        assert!(settings.data_type.is_none());
        assert!(settings.create_fn);
        assert!(settings.remove_fn);
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
        assert_type(settings.dest.1, "Page");
        assert_eq!(settings.from.0.to_string(), "user_id");
        assert_type(settings.from.1, "User");
        assert!(settings.data_type.is_none());
        assert!(settings.create_fn);
        assert!(settings.remove_fn);
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
        assert_type(settings.dest.1, "User");
        assert_eq!(settings.from.0.to_string(), "blocking_user");
        assert_type(settings.from.1, "User");
        assert!(matches!(settings.data_type, Some(Type::Path(_))));
        assert!(settings.create_fn);
        assert!(settings.remove_fn);
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
        assert_type(settings.dest.1, "Site");
        assert_eq!(settings.from.0.to_string(), "user_id");
        assert_type(settings.from.1, "User");
        assert!(matches!(settings.data_type, Some(Type::Path(_))));
        assert!(!settings.create_fn);
        assert!(settings.remove_fn);
        assert!(settings.define_struct);
    }

    {
        let settings = parse_settings(
            "
            name => UserBotOwner,
            dest => bot_user: User,
            from => owner_user: User,
            data => UserBotMetadata,
            create_fn => false,
            define_struct => false,
            ",
        );
        assert_eq!(settings.relation_name.to_string(), "UserBotOwner");
        assert_eq!(settings.field_name.to_string(), "user_bot_owner");
        assert_eq!(settings.dest.0.to_string(), "bot_user");
        assert_type(settings.dest.1, "User");
        assert_eq!(settings.from.0.to_string(), "owner_user");
        assert_type(settings.from.1, "User");
        assert!(matches!(settings.data_type, Some(Type::Path(_))));
        assert!(!settings.create_fn);
        assert!(settings.remove_fn);
        assert!(!settings.define_struct);
    }
}
