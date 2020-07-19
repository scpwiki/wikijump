<?php


namespace DB;

use BaseDBObject;

/**
 * Base class mapped to the database table petition_campaign.
 */
class PetitionCampaignBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='petition_campaign';
        $this->peerName = 'DB\\PetitionCampaignPeer';
        $this->primaryKeyName = 'campaign_id';
        $this->fieldNames = array( 'campaign_id' ,  'site_id' ,  'name' ,  'identifier' ,  'active' ,  'number_signatures' ,  'deleted' ,  'collect_address' ,  'collect_city' ,  'collect_state' ,  'collect_zip' ,  'collect_country' ,  'collect_comments' ,  'show_city' ,  'show_state' ,  'show_zip' ,  'show_country' ,  'show_comments' ,  'thank_you_page' );

        //$this->fieldDefaultValues=
    }






    public function getCampaignId()
    {
        return $this->getFieldValue('campaign_id');
    }

    public function setCampaignId($v1, $raw = false)
    {
        $this->setFieldValue('campaign_id', $v1, $raw);
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


    public function getIdentifier()
    {
        return $this->getFieldValue('identifier');
    }

    public function setIdentifier($v1, $raw = false)
    {
        $this->setFieldValue('identifier', $v1, $raw);
    }


    public function getActive()
    {
        return $this->getFieldValue('active');
    }

    public function setActive($v1, $raw = false)
    {
        $this->setFieldValue('active', $v1, $raw);
    }


    public function getNumberSignatures()
    {
        return $this->getFieldValue('number_signatures');
    }

    public function setNumberSignatures($v1, $raw = false)
    {
        $this->setFieldValue('number_signatures', $v1, $raw);
    }


    public function getDeleted()
    {
        return $this->getFieldValue('deleted');
    }

    public function setDeleted($v1, $raw = false)
    {
        $this->setFieldValue('deleted', $v1, $raw);
    }


    public function getCollectAddress()
    {
        return $this->getFieldValue('collect_address');
    }

    public function setCollectAddress($v1, $raw = false)
    {
        $this->setFieldValue('collect_address', $v1, $raw);
    }


    public function getCollectCity()
    {
        return $this->getFieldValue('collect_city');
    }

    public function setCollectCity($v1, $raw = false)
    {
        $this->setFieldValue('collect_city', $v1, $raw);
    }


    public function getCollectState()
    {
        return $this->getFieldValue('collect_state');
    }

    public function setCollectState($v1, $raw = false)
    {
        $this->setFieldValue('collect_state', $v1, $raw);
    }


    public function getCollectZip()
    {
        return $this->getFieldValue('collect_zip');
    }

    public function setCollectZip($v1, $raw = false)
    {
        $this->setFieldValue('collect_zip', $v1, $raw);
    }


    public function getCollectCountry()
    {
        return $this->getFieldValue('collect_country');
    }

    public function setCollectCountry($v1, $raw = false)
    {
        $this->setFieldValue('collect_country', $v1, $raw);
    }


    public function getCollectComments()
    {
        return $this->getFieldValue('collect_comments');
    }

    public function setCollectComments($v1, $raw = false)
    {
        $this->setFieldValue('collect_comments', $v1, $raw);
    }


    public function getShowCity()
    {
        return $this->getFieldValue('show_city');
    }

    public function setShowCity($v1, $raw = false)
    {
        $this->setFieldValue('show_city', $v1, $raw);
    }


    public function getShowState()
    {
        return $this->getFieldValue('show_state');
    }

    public function setShowState($v1, $raw = false)
    {
        $this->setFieldValue('show_state', $v1, $raw);
    }


    public function getShowZip()
    {
        return $this->getFieldValue('show_zip');
    }

    public function setShowZip($v1, $raw = false)
    {
        $this->setFieldValue('show_zip', $v1, $raw);
    }


    public function getShowCountry()
    {
        return $this->getFieldValue('show_country');
    }

    public function setShowCountry($v1, $raw = false)
    {
        $this->setFieldValue('show_country', $v1, $raw);
    }


    public function getShowComments()
    {
        return $this->getFieldValue('show_comments');
    }

    public function setShowComments($v1, $raw = false)
    {
        $this->setFieldValue('show_comments', $v1, $raw);
    }


    public function getThankYouPage()
    {
        return $this->getFieldValue('thank_you_page');
    }

    public function setThankYouPage($v1, $raw = false)
    {
        $this->setFieldValue('thank_you_page', $v1, $raw);
    }
}
