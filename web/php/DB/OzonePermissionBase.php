<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBObject;

/**
 * Base Class mapped to the database table ozone_permission.
 */
class OzonePermissionBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='ozone_permission';
        $this->peerName = 'DB\\OzonePermissionPeer';
        $this->primaryKeyName = 'permission_id';
        $this->fieldNames = array( 'permission_id' ,  'name' ,  'description' );

        //$this->fieldDefaultValues=
    }






    public function getPermissionId()
    {
        return $this->getFieldValue('permission_id');
    }

    public function setPermissionId($v1, $raw = false)
    {
        $this->setFieldValue('permission_id', $v1, $raw);
    }


    public function getName()
    {
        return $this->getFieldValue('name');
    }

    public function setName($v1, $raw = false)
    {
        $this->setFieldValue('name', $v1, $raw);
    }


    public function getDescription()
    {
        return $this->getFieldValue('description');
    }

    public function setDescription($v1, $raw = false)
    {
        $this->setFieldValue('description', $v1, $raw);
    }
}
