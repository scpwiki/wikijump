<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBObject;

/**
 * Base Class mapped to the database table license.
 */
class LicenseBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='license';
        $this->peerName = 'Wikidot\\DB\\LicensePeer';
        $this->primaryKeyName = 'license_id';
        $this->fieldNames = array( 'license_id' ,  'name' ,  'description' ,  'sort' );

        //$this->fieldDefaultValues=
    }






    public function getLicenseId()
    {
        return $this->getFieldValue('license_id');
    }

    public function setLicenseId($v1, $raw = false)
    {
        $this->setFieldValue('license_id', $v1, $raw);
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


    public function getSort()
    {
        return $this->getFieldValue('sort');
    }

    public function setSort($v1, $raw = false)
    {
        $this->setFieldValue('sort', $v1, $raw);
    }
}
