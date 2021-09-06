<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBObject;

/**
 * Base Class mapped to the database table site.
 */
class SiteBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='site';
        $this->peerName = 'Wikidot\\DB\\SitePeer';
        $this->primaryKeyName = 'site_id';
        $this->fieldNames = array( 'site_id' ,  'name' ,  'subtitle' ,  'unix_name' ,  'description' ,  'language' ,  'date_created' ,  'custom_domain' ,  'default_page' ,  'visible' ,  'private' ,  'deleted' , 'enable_allowed_tags' );

        //$this->fieldDefaultValues=
    }






    public function getSiteId()
    {
        return $this->getFieldValue('site_id');
    }

    public function setSiteId($v1, $raw = false)
    {
        $this->setFieldValue('site_id', $v1, $raw);
    }


    public function getName()
    {
        return $this->getFieldValue('name');
    }

    public function setName($v1, $raw = false)
    {
        $this->setFieldValue('name', $v1, $raw);
    }


    public function getSubtitle()
    {
        return $this->getFieldValue('subtitle');
    }

    public function setSubtitle($v1, $raw = false)
    {
        $this->setFieldValue('subtitle', $v1, $raw);
    }


    public function getUnixName()
    {
        return $this->getFieldValue('unix_name');
    }

    public function setUnixName($v1, $raw = false)
    {
        $this->setFieldValue('unix_name', $v1, $raw);
    }


    public function getDescription()
    {
        return $this->getFieldValue('description');
    }

    public function setDescription($v1, $raw = false)
    {
        $this->setFieldValue('description', $v1, $raw);
    }


    public function getLanguage()
    {
        return $this->getFieldValue('language');
    }

    public function setLanguage($v1, $raw = false)
    {
        $this->setFieldValue('language', $v1, $raw);
    }


    public function getDateCreated()
    {
        return $this->getFieldValue('date_created');
    }

    public function setDateCreated($v1, $raw = false)
    {
        $this->setFieldValue('date_created', $v1, $raw);
    }


    public function getCustomDomain()
    {
        return $this->getFieldValue('custom_domain');
    }

    public function setCustomDomain($v1, $raw = false)
    {
        $this->setFieldValue('custom_domain', $v1, $raw);
    }


    public function getDefaultPage()
    {
        return $this->getFieldValue('default_page');
    }

    public function setDefaultPage($v1, $raw = false)
    {
        $this->setFieldValue('default_page', $v1, $raw);
    }


    public function getVisible()
    {
        return $this->getFieldValue('visible');
    }

    public function setVisible($v1, $raw = false)
    {
        $this->setFieldValue('visible', $v1, $raw);
    }


    public function getPrivate()
    {
        return $this->getFieldValue('private');
    }

    public function setPrivate($v1, $raw = false)
    {
        $this->setFieldValue('private', $v1, $raw);
    }


    public function getDeleted()
    {
        return $this->getFieldValue('deleted');
    }

    public function setDeleted($v1, $raw = false)
    {
        $this->setFieldValue('deleted', $v1, $raw);
    }


    public function getEnableAllowedTags()
    {
        return $this->getFieldValue('enable_allowed_tags');
    }

    public function setEnableAllowedTags($v1, $raw = false)
    {
        $this->setFieldValue('enable_allowed_tags', $v1, $raw);
    }
}
