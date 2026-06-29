INSERT INTO role_permission (
    role_id,
    site_id,
    resource_type,
    resource_category_id,
    action
)
-- There is no persisted stable seeded-role identifier in existing databases.
-- Preserve upgrade behavior for existing role managers by backfilling Role:Edit
-- to active roles that already had Role:Assign before this permission split.
SELECT DISTINCT
    role_permission.role_id,
    role_permission.site_id,
    'role',
    NULL::BIGINT,
    'edit'
FROM role_permission
INNER JOIN role
    ON role.role_id = role_permission.role_id
    AND role.site_id = role_permission.site_id
WHERE
    role_permission.resource_type = 'role'
    AND role_permission.resource_category_id IS NULL
    AND role_permission.action = 'assign'
    AND role.deleted_at IS NULL
    AND NOT EXISTS (
        SELECT 1
        FROM role_permission existing_role_edit
        WHERE
            existing_role_edit.role_id = role_permission.role_id
            AND existing_role_edit.site_id = role_permission.site_id
            AND existing_role_edit.resource_type = 'role'
            AND existing_role_edit.resource_category_id IS NULL
            AND existing_role_edit.action = 'edit'
    );
