<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBObject;

/**
 * Base Class mapped to the database table moderator.
 */
class ModeratorBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='moderator';
        $this->peerName = 'Wikidot\\DB\\ModeratorPeer';
        $this->primaryKeyName = 'moderator_id';
        $this->fieldNames = array( 'moderator_id' ,  'site_id' ,  'user_id' ,  'permissions' );

        //$this->fieldDefaultValues=
    }






    public function getModeratorId()
    {
        return $this->getFieldValue('moderator_id');
    }

    public function setModeratorId($v1, $raw = false)
    {
        $this->setFieldValue('moderator_id', $v1, $raw);
    }


    public function getSiteId()
    {
        return $this->getFieldValue('site_id');
    }

    public function setSiteId($v1, $raw = false)
    {
        $this->setFieldValue('site_id', $v1, $raw);
    }


    public function getUserId()
    {
        return $this->getFieldValue('user_id');
    }

    public function setUserId($v1, $raw = false)
    {
        $this->setFieldValue('user_id', $v1, $raw);
    }


    public function getPermissions()
    {
        return $this->getFieldValue('permissions');
    }

    public function setPermissions($v1, $raw = false)
    {
        $this->setFieldValue('permissions', $v1, $raw);
    }
}
