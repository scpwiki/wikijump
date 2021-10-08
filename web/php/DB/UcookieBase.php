<?php

namespace Wikidot\DB;



use Ozone\Framework\DB\OzoneSession;
use Ozone\Framework\Database\BaseDBObject;
use Ozone\Framework\Database\Criteria;

/**
 * Base Class mapped to the database table ucookie.
 */
class UcookieBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='ucookie';
        $this->peerName = 'Wikidot\\DB\\UcookiePeer';
        $this->primaryKeyName = 'ucookie_id';
        $this->fieldNames = array( 'ucookie_id' ,  'site_id' ,  'session_id' ,  'date_granted' );
    }



    public function getSite()
    {
        if (is_array($this->prefetched)) {
            if (in_array('site', $this->prefetched)) {
                if (in_array('site', $this->prefetchedObjects)) {
                    return $this->prefetchedObjects['site'];
                } else {
                    $obj = new Site($this->sourceRow);
                    $obj->setNew(false);
                    //$obj->prefetched = $this->prefetched;
                    //$obj->sourceRow = $this->sourceRow;
                    $this->prefetchedObjects['site'] = $obj;
                    return $obj;
                }
            }
        }
                $foreignPeerClassName = 'Wikidot\\DB\\SitePeer';
                $fpeer = new $foreignPeerClassName();

                $criteria = new Criteria();

                $criteria->add("site_id", $this->fieldValues['site_id']);

                $result = $fpeer->selectOneByCriteria($criteria);
                return $result;
    }

    public function setSite($primaryObject)
    {
        $this->fieldValues['site_id'] = $primaryObject->getFieldValue('site_id');
    }


    public function getSiteId()
    {
        return $this->getFieldValue('site_id');
    }

    public function setSiteId($v1, $raw = false)
    {
        $this->setFieldValue('site_id', $v1, $raw);
    }


    public function getSessionId()
    {
        return $this->getFieldValue('session_id');
    }

    public function setSessionId($v1, $raw = false)
    {
        $this->setFieldValue('session_id', $v1, $raw);
    }


}
