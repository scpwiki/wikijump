<?php


namespace DB;

use BaseDBObject;

/**
 * Base class mapped to the database table ozone_group.
 */
class OzoneGroupBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='ozone_group';
        $this->peerName = 'DB\\OzoneGroupPeer';
        $this->primaryKeyName = 'group_id';
        $this->fieldNames = array( 'group_id' ,  'parent_group_id' ,  'name' ,  'description' );

        //$this->fieldDefaultValues=
    }






    public function getGroupId()
    {
        return $this->getFieldValue('group_id');
    }

    public function setGroupId($v1, $raw = false)
    {
        $this->setFieldValue('group_id', $v1, $raw);
    }


    public function getParentGroupId()
    {
        return $this->getFieldValue('parent_group_id');
    }

    public function setParentGroupId($v1, $raw = false)
    {
        $this->setFieldValue('parent_group_id', $v1, $raw);
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
