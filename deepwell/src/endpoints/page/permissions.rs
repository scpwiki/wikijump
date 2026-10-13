//! Page endpoint permission checks.

use super::*;

pub(super) async fn ensure_page_create_permission(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    slug: &str,
    user_id: i64,
) -> Result<()> {
    let category_slug = get_category_name(slug);
    let category = CategoryService::get_optional(
        ctx,
        site_id,
        Reference::Slug(Cow::Borrowed(category_slug)),
    )
    .await
    .or_raise(|| {
        Error::new(
            "failed to load page category for create permission check",
            ErrorType::Permission,
        )
    })?;

    let resource_category = category.map(|category| Reference::Id(category.category_id));
    ensure_page_permission(
        ctx,
        site_id,
        None,
        resource_category,
        user_id,
        Action::Create,
        "create",
    )
    .await
}

pub(super) async fn ensure_page_edit_permission<'a>(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    page_reference: Reference<'a>,
    user_id: i64,
) -> Result<()> {
    ensure_page_action_permission(
        ctx,
        site_id,
        page_reference,
        user_id,
        Action::Edit,
        "edit",
    )
    .await
}

pub(super) async fn ensure_page_action_permission<'a>(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    page_reference: Reference<'a>,
    user_id: i64,
    action: Action,
    action_name: &str,
) -> Result<()> {
    let page = PageService::get(ctx, site_id, page_reference.clone())
        .await
        .or_raise(|| {
            Error::new(
                format!("failed to load page for {action_name} permission check"),
                ErrorType::Permission,
            )
        })?;

    ensure_page_permission(
        ctx,
        site_id,
        Some(Reference::Id(page.page_id)),
        Some(Reference::Id(page.page_category_id)),
        user_id,
        action,
        action_name,
    )
    .await
}

pub(super) async fn ensure_deleted_page_delete_permission_and_get_category_id(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    page_id: i64,
    user_id: i64,
) -> Result<i64> {
    let page = PageService::get_direct(ctx, page_id, true)
        .await
        .or_raise(|| {
            Error::new(
                "failed to load deleted page for delete permission check",
                ErrorType::Permission,
            )
        })?;

    if page.site_id != site_id {
        return Err(Error::new(
            "deleted page is not in the requested site",
            ErrorType::PermissionDenied,
        )
        .into());
    }

    ensure_page_permission(
        ctx,
        site_id,
        Some(Reference::Id(page.page_id)),
        Some(Reference::Id(page.page_category_id)),
        user_id,
        Action::Delete,
        "delete",
    )
    .await?;

    Ok(page.page_category_id)
}

pub(super) async fn ensure_page_permission<'a>(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    page_reference: Option<Reference<'a>>,
    resource_category: Option<Reference<'a>>,
    user_id: i64,
    action: Action,
    action_name: &str,
) -> Result<()> {
    let can_mutate = PermissionService::check_user_can(
        ctx,
        &CheckPermissionContext {
            user_id: Some(user_id),
            site_id,
            page_reference,
        },
        Permission {
            resource_type: Resource::Page,
            resource_category,
            action,
        },
    )
    .await
    .or_raise(|| Error::new("failed to check page permission", ErrorType::Permission))?;

    if can_mutate {
        Ok(())
    } else {
        Err(Error::new(
            format!("user does not have permission to {action_name} this page"),
            ErrorType::PermissionDenied,
        )
        .into())
    }
}

pub(super) async fn ensure_page_view_permission(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    page_id: i64,
) -> Result<PageModel> {
    let page = PageService::get(ctx, site_id, Reference::Id(page_id))
        .await
        .or_raise(|| {
            Error::new(
                "failed to load page for view permission check",
                ErrorType::Permission,
            )
        })?;

    let can_view =
        PermissionService::check_user_can_view_page(ctx, ctx.request().user_id, &page)
            .await
            .or_raise(|| {
                Error::new(
                    "failed to check page view permission",
                    ErrorType::Permission,
                )
            })?;

    if can_view {
        Ok(page)
    } else {
        Err(Error::new(
            "user does not have permission to view this page",
            ErrorType::PermissionDenied,
        )
        .into())
    }
}

pub async fn page_edit_permission(
    ctx: &ServiceContext<'_>,
    _params: Params<'static>,
) -> Result<PageEditPermissionOutput> {
    let can_edit = PageService::check_user_permission(ctx, Action::Edit)
        .await
        .or_raise(|| {
            Error::new("failed to check page edit permission", ErrorType::Page)
        })?;

    Ok(PageEditPermissionOutput { can_edit })
}

pub async fn page_move_permission(
    ctx: &ServiceContext<'_>,
    _params: Params<'static>,
) -> Result<PageMovePermissionOutput> {
    let can_move = PageService::check_user_permission(ctx, Action::Rename)
        .await
        .or_raise(|| {
            Error::new("failed to check page move permission", ErrorType::Page)
        })?;

    Ok(PageMovePermissionOutput { can_move })
}

pub async fn page_delete_permission(
    ctx: &ServiceContext<'_>,
    _params: Params<'static>,
) -> Result<PageDeletePermissionOutput> {
    let can_delete = PageService::check_user_permission(ctx, Action::Delete)
        .await
        .or_raise(|| {
            Error::new("failed to check page delete permission", ErrorType::Page)
        })?;

    Ok(PageDeletePermissionOutput { can_delete })
}
