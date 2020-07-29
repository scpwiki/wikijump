<?php


namespace DB;

use BaseDBObject;

/**
 * Base class mapped to the database table forum_group.
 */
class ForumGroupBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='forum_group';
        $this->peerName = 'DB\\ForumGroupPeer';
        $this->primaryKeyName = 'group_id';
        $this->fieldNames = array( 'group_id' ,  'name' ,  'description' ,  'sort_index' ,  'site_id' ,  'visible' );

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


    public function getSortIndex()
    {
        return $this->getFieldValue('sort_index');
    }

    public function setSortIndex($v1, $raw = false)
    {
        $this->setFieldValue('sort_index', $v1, $raw);
    }


    public function getSiteId()
    {
        return $this->getFieldValue('site_id');
    }

    public function setSiteId($v1, $raw = false)
    {
        $this->setFieldValue('site_id', $v1, $raw);
    }


    public function getVisible()
    {
        return $this->getFieldValue('visible');
    }

    public function setVisible($v1, $raw = false)
    {
        $this->setFieldValue('visible', $v1, $raw);
    }
}
