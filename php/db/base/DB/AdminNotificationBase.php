<?php


namespace DB;

use BaseDBObject;

/**
 * Base class mapped to the database table admin_notification.
 */
class AdminNotificationBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='admin_notification';
        $this->peerName = 'DB\\AdminNotificationPeer';
        $this->primaryKeyName = 'notification_id';
        $this->fieldNames = array( 'notification_id' ,  'site_id' ,  'body' ,  'type' ,  'viewed' ,  'date' ,  'extra' ,  'notify_online' ,  'notify_feed' ,  'notify_email' );

        //$this->fieldDefaultValues=
    }






    public function getNotificationId()
    {
        return $this->getFieldValue('notification_id');
    }

    public function setNotificationId($v1, $raw = false)
    {
        $this->setFieldValue('notification_id', $v1, $raw);
    }


    public function getSiteId()
    {
        return $this->getFieldValue('site_id');
    }

    public function setSiteId($v1, $raw = false)
    {
        $this->setFieldValue('site_id', $v1, $raw);
    }


    public function getBody()
    {
        return $this->getFieldValue('body');
    }

    public function setBody($v1, $raw = false)
    {
        $this->setFieldValue('body', $v1, $raw);
    }


    public function getType()
    {
        return $this->getFieldValue('type');
    }

    public function setType($v1, $raw = false)
    {
        $this->setFieldValue('type', $v1, $raw);
    }


    public function getViewed()
    {
        return $this->getFieldValue('viewed');
    }

    public function setViewed($v1, $raw = false)
    {
        $this->setFieldValue('viewed', $v1, $raw);
    }


    public function getDate()
    {
        return $this->getFieldValue('date');
    }

    public function setDate($v1, $raw = false)
    {
        $this->setFieldValue('date', $v1, $raw);
    }


    public function getExtra()
    {
        return $this->getFieldValue('extra');
    }

    public function setExtra($v1, $raw = false)
    {
        $this->setFieldValue('extra', $v1, $raw);
    }


    public function getNotifyOnline()
    {
        return $this->getFieldValue('notify_online');
    }

    public function setNotifyOnline($v1, $raw = false)
    {
        $this->setFieldValue('notify_online', $v1, $raw);
    }


    public function getNotifyFeed()
    {
        return $this->getFieldValue('notify_feed');
    }

    public function setNotifyFeed($v1, $raw = false)
    {
        $this->setFieldValue('notify_feed', $v1, $raw);
    }


    public function getNotifyEmail()
    {
        return $this->getFieldValue('notify_email');
    }

    public function setNotifyEmail($v1, $raw = false)
    {
        $this->setFieldValue('notify_email', $v1, $raw);
    }
}
