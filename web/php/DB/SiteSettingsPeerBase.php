<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBPeer;

/**
 * Base peer Class mapped to the database table site_settings.
 */
class SiteSettingsPeerBase extends BaseDBPeer
{
    public static $peerInstance;

    protected function internalInit()
    {
        $this->tableName='site_settings';
        $this->objectName='Wikidot\\DB\\SiteSettings';
        $this->primaryKeyName = 'site_id';
        $this->fieldNames = array( 'site_id' ,  'allow_membership_by_apply' ,  'allow_membership_by_password' ,  'membership_password' ,  'private_landing_page' ,  'hide_navigation_unauthorized' ,  'max_private_members' ,  'max_private_viewers' ,  'ssl_mode' ,  'file_storage_size' ,  'max_upload_file_size' ,  'openid_enabled' ,  'allow_members_invite' ,  'enable_all_pingback_out' );
        $this->fieldTypes = array( 'site_id' => 'int',  'allow_membership_by_apply' => 'boolean',  'allow_membership_by_password' => 'boolean',  'membership_password' => 'varchar(80)',  'private_landing_page' => 'varchar(80)',  'hide_navigation_unauthorized' => 'boolean',  'max_private_members' => 'int',  'max_private_viewers' => 'int',  'ssl_mode' => 'varchar(20)',  'file_storage_size' => 'int',  'max_upload_file_size' => 'int',  'openid_enabled' => 'boolean',  'allow_members_invite' => 'boolean',  'enable_all_pingback_out' => 'boolean');
        $this->defaultValues = array( 'allow_membership_by_apply' => 'true',  'allow_membership_by_password' => 'false',  'private_landing_page' => 'system:join',  'hide_navigation_unauthorized' => 'true',  'max_private_members' => '50',  'max_private_viewers' => '20',  'file_storage_size' => '314572800',  'max_upload_file_size' => '10485760',  'openid_enabled' => 'false',  'allow_members_invite' => 'false',  'enable_all_pingback_out' => 'true');
    }

    public static function instance()
    {
        if (self::$peerInstance == null) {
            self::$peerInstance = new SiteSettingsPeer();
        }
        return self::$peerInstance;
    }
}
