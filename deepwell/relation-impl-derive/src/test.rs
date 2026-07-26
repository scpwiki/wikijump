use crate::parse::RelationSettings;

/// Validate that `RelationSettings` are being parsed as expected.
#[test]
fn parse() {
    let settings: RelationSettings = syn::parse_str(
        "
        name => UserBlock,
        dest => blocked_user: User,
        from => blocking_user: User,
        data => UserBlockData,
        ",
    )
    .expect("failed to parse macro");

    todo!()
}

/// Validate that code generation is as expected given some `RelationSettings`.
#[test]
fn generate() {
    // TODO
}
