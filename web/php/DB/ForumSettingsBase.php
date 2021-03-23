<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBObject;

/**
 * Base Class mapped to the database table forum_settings.
 */
class ForumSettingsBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='forum_settings';
        $this->peerName = 'DB\\ForumSettingsPeer';
        $this->primaryKeyName = 'site_id';
        $this->fieldNames = array( 'site_id' ,  'permissions' ,  'per_page_discussion' ,  'max_nest_level' );

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


    public function getPermissions()
    {
        return $this->getFieldValue('permissions');
    }

    public function setPermissions($v1, $raw = false)
    {
        $this->setFieldValue('permissions', $v1, $raw);
    }


    public function getPerPageDiscussion()
    {
        return $this->getFieldValue('per_page_discussion');
    }

    public function setPerPageDiscussion($v1, $raw = false)
    {
        $this->setFieldValue('per_page_discussion', $v1, $raw);
    }


    public function getMaxNestLevel()
    {
        return $this->getFieldValue('max_nest_level');
    }

    public function setMaxNestLevel($v1, $raw = false)
    {
        $this->setFieldValue('max_nest_level', $v1, $raw);
    }
}
