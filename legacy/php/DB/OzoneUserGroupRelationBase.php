<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBObject;
use Ozone\Framework\Database\Criteria;
use Wikijump\Models\User;

/**
 * Base Class mapped to the database table ozone_user_group_relation.
 */
class OzoneUserGroupRelationBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='ozone_user_group_relation';
        $this->peerName = 'Wikidot\\DB\\OzoneUserGroupRelationPeer';
        $this->primaryKeyName = 'user_group_id';
        $this->fieldNames = array( 'user_group_id' ,  'user_id' ,  'group_id' );

        //$this->fieldDefaultValues=
    }



    public function getOzoneUser()
    {
        if (is_array($this->prefetched)) {
            if (in_array('ozone_user', $this->prefetched)) {
                if (in_array('ozone_user', $this->prefetchedObjects)) {
                    return $this->prefetchedObjects['ozone_user'];
                } else {
                    $obj = User::find($this->sourceRow);
                    $this->prefetchedObjects['ozone_user'] = $obj;
                    return $obj;
                }
            }
        }

        $result = User::find($this->fieldValues['user_id']);
        return $result;
    }

    public function setOzoneUser($primaryObject)
    {
        $this->fieldValues['user_id'] = $primaryObject->getFieldValue('user_id');
    }
    public function getOzoneGroup()
    {
        if (is_array($this->prefetched)) {
            if (in_array('ozone_group', $this->prefetched)) {
                if (in_array('ozone_group', $this->prefetchedObjects)) {
                    return $this->prefetchedObjects['ozone_group'];
                } else {
                    $obj = new OzoneGroup($this->sourceRow);
                    $obj->setNew(false);
                    //$obj->prefetched = $this->prefetched;
                    //$obj->sourceRow = $this->sourceRow;
                    $this->prefetchedObjects['ozone_group'] = $obj;
                    return $obj;
                }
            }
        }
                $foreignPeerClassName = 'Wikidot\\DB\\OzoneGroupPeer';
                $fpeer = new $foreignPeerClassName();

                $criteria = new Criteria();

                $criteria->add("group_id", $this->fieldValues['group_id']);

                $result = $fpeer->selectOneByCriteria($criteria);
                return $result;
    }

    public function setOzoneGroup($primaryObject)
    {
        $this->fieldValues['group_id'] = $primaryObject->getFieldValue('group_id');
    }



    public function getUserGroupId()
    {
        return $this->getFieldValue('user_group_id');
    }

    public function setUserGroupId($v1, $raw = false)
    {
        $this->setFieldValue('user_group_id', $v1, $raw);
    }


    public function getUserId()
    {
        return $this->getFieldValue('user_id');
    }

    public function setUserId($v1, $raw = false)
    {
        $this->setFieldValue('user_id', $v1, $raw);
    }


    public function getGroupId()
    {
        return $this->getFieldValue('group_id');
    }

    public function setGroupId($v1, $raw = false)
    {
        $this->setFieldValue('group_id', $v1, $raw);
    }
}
