<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBObject;

/**
 * Base Class mapped to the database table profile.
 */
class ProfileBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='profile';
        $this->peerName = 'Wikidot\\DB\\ProfilePeer';
        $this->primaryKeyName = 'user_id';
        $this->fieldNames = array( 'user_id' ,  'real_name' ,  'gender' ,  'birthday_day' ,  'birthday_month' ,  'birthday_year' ,  'about' ,  'location' ,  'website' ,  'im_aim' ,  'im_gadu_gadu' ,  'im_google_talk' ,  'im_icq' ,  'im_jabber' ,  'im_msn' ,  'im_yahoo' ,  'change_screen_name_count' );

        //$this->fieldDefaultValues=
    }






    public function getUserId()
    {
        return $this->getFieldValue('user_id');
    }

    public function setUserId($v1, $raw = false)
    {
        $this->setFieldValue('user_id', $v1, $raw);
    }


    public function getRealName()
    {
        return $this->getFieldValue('real_name');
    }

    public function setRealName($v1, $raw = false)
    {
        $this->setFieldValue('real_name', $v1, $raw);
    }


    public function getGender()
    {
        return $this->getFieldValue('gender');
    }

    public function setGender($v1, $raw = false)
    {
        $this->setFieldValue('gender', $v1, $raw);
    }


    public function getBirthdayDay()
    {
        return $this->getFieldValue('birthday_day');
    }

    public function setBirthdayDay($v1, $raw = false)
    {
        $this->setFieldValue('birthday_day', $v1, $raw);
    }


    public function getBirthdayMonth()
    {
        return $this->getFieldValue('birthday_month');
    }

    public function setBirthdayMonth($v1, $raw = false)
    {
        $this->setFieldValue('birthday_month', $v1, $raw);
    }


    public function getBirthdayYear()
    {
        return $this->getFieldValue('birthday_year');
    }

    public function setBirthdayYear($v1, $raw = false)
    {
        $this->setFieldValue('birthday_year', $v1, $raw);
    }


    public function getAbout()
    {
        return $this->getFieldValue('about');
    }

    public function setAbout($v1, $raw = false)
    {
        $this->setFieldValue('about', $v1, $raw);
    }


    public function getLocation()
    {
        return $this->getFieldValue('location');
    }

    public function setLocation($v1, $raw = false)
    {
        $this->setFieldValue('location', $v1, $raw);
    }


    public function getWebsite()
    {
        return $this->getFieldValue('website');
    }

    public function setWebsite($v1, $raw = false)
    {
        $this->setFieldValue('website', $v1, $raw);
    }


    public function getImAim()
    {
        return $this->getFieldValue('im_aim');
    }

    public function setImAim($v1, $raw = false)
    {
        $this->setFieldValue('im_aim', $v1, $raw);
    }


    public function getImGaduGadu()
    {
        return $this->getFieldValue('im_gadu_gadu');
    }

    public function setImGaduGadu($v1, $raw = false)
    {
        $this->setFieldValue('im_gadu_gadu', $v1, $raw);
    }


    public function getImGoogleTalk()
    {
        return $this->getFieldValue('im_google_talk');
    }

    public function setImGoogleTalk($v1, $raw = false)
    {
        $this->setFieldValue('im_google_talk', $v1, $raw);
    }


    public function getImIcq()
    {
        return $this->getFieldValue('im_icq');
    }

    public function setImIcq($v1, $raw = false)
    {
        $this->setFieldValue('im_icq', $v1, $raw);
    }


    public function getImJabber()
    {
        return $this->getFieldValue('im_jabber');
    }

    public function setImJabber($v1, $raw = false)
    {
        $this->setFieldValue('im_jabber', $v1, $raw);
    }


    public function getImMsn()
    {
        return $this->getFieldValue('im_msn');
    }

    public function setImMsn($v1, $raw = false)
    {
        $this->setFieldValue('im_msn', $v1, $raw);
    }


    public function getImYahoo()
    {
        return $this->getFieldValue('im_yahoo');
    }

    public function setImYahoo($v1, $raw = false)
    {
        $this->setFieldValue('im_yahoo', $v1, $raw);
    }


    public function getChangeScreenNameCount()
    {
        return $this->getFieldValue('change_screen_name_count');
    }

    public function setChangeScreenNameCount($v1, $raw = false)
    {
        $this->setFieldValue('change_screen_name_count', $v1, $raw);
    }
}
