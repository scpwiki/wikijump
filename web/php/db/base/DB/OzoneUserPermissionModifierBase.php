<?php


namespace DB;

use BaseDBObject;
use Criteria;

/**
 * Base class mapped to the database table ozone_user_permission_modifier.
 */
class OzoneUserPermissionModifierBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='ozone_user_permission_modifier';
        $this->peerName = 'DB\\OzoneUserPermissionModifierPeer';
        $this->primaryKeyName = 'user_permission_id';
        $this->fieldNames = array( 'user_permission_id' ,  'user_id' ,  'permission_id' ,  'modifier' );

        //$this->fieldDefaultValues=
    }



    public function getOzoneUser()
    {
        if (is_array($this->prefetched)) {
            if (in_array('ozone_user', $this->prefetched)) {
                if (in_array('ozone_user', $this->prefetchedObjects)) {
                    return $this->prefetchedObjects['ozone_user'];
                } else {
                    $obj = new OzoneUser($this->sourceRow);
                    $obj->setNew(false);
                    //$obj->prefetched = $this->prefetched;
                    //$obj->sourceRow = $this->sourceRow;
                    $this->prefetchedObjects['ozone_user'] = $obj;
                    return $obj;
                }
            }
        }
                $foreignPeerClassName = 'DB\\OzoneUserPeer';
                $fpeer = new $foreignPeerClassName();

                $criteria = new Criteria();

                $criteria->add("", $this->fieldValues['user_id']);

                $result = $fpeer->selectOneByCriteria($criteria);
                return $result;
    }

    public function setOzoneUser($primaryObject)
    {
        $this->fieldValues['user_id'] = $primaryObject->getFieldValue('');
    }
    public function getOzonePermission()
    {
        if (is_array($this->prefetched)) {
            if (in_array('ozone_permission', $this->prefetched)) {
                if (in_array('ozone_permission', $this->prefetchedObjects)) {
                    return $this->prefetchedObjects['ozone_permission'];
                } else {
                    $obj = new OzonePermission($this->sourceRow);
                    $obj->setNew(false);
                    //$obj->prefetched = $this->prefetched;
                    //$obj->sourceRow = $this->sourceRow;
                    $this->prefetchedObjects['ozone_permission'] = $obj;
                    return $obj;
                }
            }
        }
                $foreignPeerClassName = 'DB\\OzonePermissionPeer';
                $fpeer = new $foreignPeerClassName();

                $criteria = new Criteria();

                $criteria->add("permission_id", $this->fieldValues['permission_id']);

                $result = $fpeer->selectOneByCriteria($criteria);
                return $result;
    }

    public function setOzonePermission($primaryObject)
    {
        $this->fieldValues['permission_id'] = $primaryObject->getFieldValue('permission_id');
    }



    public function getUserPermissionId()
    {
        return $this->getFieldValue('user_permission_id');
    }

    public function setUserPermissionId($v1, $raw = false)
    {
        $this->setFieldValue('user_permission_id', $v1, $raw);
    }


    public function getUserId()
    {
        return $this->getFieldValue('user_id');
    }

    public function setUserId($v1, $raw = false)
    {
        $this->setFieldValue('user_id', $v1, $raw);
    }


    public function getPermissionId()
    {
        return $this->getFieldValue('permission_id');
    }

    public function setPermissionId($v1, $raw = false)
    {
        $this->setFieldValue('permission_id', $v1, $raw);
    }


    public function getModifier()
    {
        return $this->getFieldValue('modifier');
    }

    public function setModifier($v1, $raw = false)
    {
        $this->setFieldValue('modifier', $v1, $raw);
    }
}
