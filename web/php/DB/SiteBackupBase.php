<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBObject;

/**
 * Base Class mapped to the database table site_backup.
 */
class SiteBackupBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='site_backup';
        $this->peerName = 'DB\\SiteBackupPeer';
        $this->primaryKeyName = 'backup_id';
        $this->fieldNames = array( 'backup_id' ,  'site_id' ,  'status' ,  'backup_source' ,  'backup_files' ,  'date' ,  'rand' );

        //$this->fieldDefaultValues=
    }






    public function getBackupId()
    {
        return $this->getFieldValue('backup_id');
    }

    public function setBackupId($v1, $raw = false)
    {
        $this->setFieldValue('backup_id', $v1, $raw);
    }


    public function getSiteId()
    {
        return $this->getFieldValue('site_id');
    }

    public function setSiteId($v1, $raw = false)
    {
        $this->setFieldValue('site_id', $v1, $raw);
    }


    public function getStatus()
    {
        return $this->getFieldValue('status');
    }

    public function setStatus($v1, $raw = false)
    {
        $this->setFieldValue('status', $v1, $raw);
    }


    public function getBackupSource()
    {
        return $this->getFieldValue('backup_source');
    }

    public function setBackupSource($v1, $raw = false)
    {
        $this->setFieldValue('backup_source', $v1, $raw);
    }


    public function getBackupFiles()
    {
        return $this->getFieldValue('backup_files');
    }

    public function setBackupFiles($v1, $raw = false)
    {
        $this->setFieldValue('backup_files', $v1, $raw);
    }


    public function getDate()
    {
        return $this->getFieldValue('date');
    }

    public function setDate($v1, $raw = false)
    {
        $this->setFieldValue('date', $v1, $raw);
    }


    public function getRand()
    {
        return $this->getFieldValue('rand');
    }

    public function setRand($v1, $raw = false)
    {
        $this->setFieldValue('rand', $v1, $raw);
    }
}
