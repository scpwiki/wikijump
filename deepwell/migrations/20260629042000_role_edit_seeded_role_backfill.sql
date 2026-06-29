INSERT INTO role_permission (
    role_id,
    site_id,
    resource_type,
    resource_category_id,
    action
)
SELECT
    role.role_id,
    role.site_id,
    'role',
    NULL,
    'edit'
FROM role
WHERE
    role.name IN ('root', 'admin')
    AND role.deleted_at IS NULL
    AND NOT EXISTS (
        SELECT 1
        FROM role_permission
        WHERE
            role_permission.role_id = role.role_id
            AND role_permission.site_id = role.site_id
            AND role_permission.resource_type = 'role'
            AND role_permission.resource_category_id IS NULL
            AND role_permission.action = 'edit'
    );
