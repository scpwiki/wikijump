<?php


namespace DB;

use BaseDBObject;

/**
 * Base class mapped to the database table site_super_settings.
 */
class SiteSuperSettingsBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='site_super_settings';
        $this->peerName = 'DB\\SiteSuperSettingsPeer';
        $this->primaryKeyName = 'site_id';
        $this->fieldNames = array( 'site_id' ,  'can_custom_domain' );

        //$this->fieldDefaultValues=
    }






    public function getSiteId()
    {
        return $this->getFieldValue('site_id');
    }

    public function setSiteId($v1, $raw = false)
    {
        $this->setFieldValue('site_id', $v1, $raw);
    }


    public function getCanCustomDomain()
    {
        return $this->getFieldValue('can_custom_domain');
    }

    public function setCanCustomDomain($v1, $raw = false)
    {
        $this->setFieldValue('can_custom_domain', $v1, $raw);
    }
}
