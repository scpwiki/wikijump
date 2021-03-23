<?php

namespace Wikidot\DB;




use Ozone\Framework\Database\BaseDBObject;

/**
 * Base Class mapped to the database table ozone_user.
 */
class OzoneUserBase extends BaseDBObject
{

    protected function internalInit()
    {
        $this->tableName='ozone_user';
        $this->peerName = 'DB\\OzoneUserPeer';
        $this->primaryKeyName = 'user_id';
        $this->fieldNames = array( 'user_id' ,  'name' ,  'nick_name' ,  'password' ,  'email' ,  'unix_name' ,  'last_login' ,  'registered_date' ,  'super_admin' ,  'super_moderator' ,  'language' );

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


    public function getName()
    {
        return $this->getFieldValue('name');
    }

    public function setName($v1, $raw = false)
    {
        $this->setFieldValue('name', $v1, $raw);
    }


    public function getNickName()
    {
        return $this->getFieldValue('nick_name');
    }

    public function setNickName($v1, $raw = false)
    {
        $this->setFieldValue('nick_name', $v1, $raw);
    }


    public function getPassword()
    {
        return $this->getFieldValue('password');
    }

    public function setPassword($v1, $raw = false)
    {
        $password = password_hash($v1, PASSWORD_BCRYPT, ['cost' => 11]);
        $this->setFieldValue('password', $password, $raw);
    }


    public function getEmail()
    {
        return $this->getFieldValue('email');
    }

    public function setEmail($v1, $raw = false)
    {
        $this->setFieldValue('email', $v1, $raw);
    }


    public function getUnixName()
    {
        return $this->getFieldValue('unix_name');
    }

    public function setUnixName($v1, $raw = false)
    {
        $this->setFieldValue('unix_name', $v1, $raw);
    }


    public function getLastLogin()
    {
        return $this->getFieldValue('last_login');
    }

    public function setLastLogin($v1, $raw = false)
    {
        $this->setFieldValue('last_login', $v1, $raw);
    }


    public function getRegisteredDate()
    {
        return $this->getFieldValue('registered_date');
    }

    public function setRegisteredDate($v1, $raw = false)
    {
        $this->setFieldValue('registered_date', $v1, $raw);
    }


    public function getSuperAdmin()
    {
        return $this->getFieldValue('super_admin');
    }

    public function setSuperAdmin($v1, $raw = false)
    {
        $this->setFieldValue('super_admin', $v1, $raw);
    }


    public function getSuperModerator()
    {
        return $this->getFieldValue('super_moderator');
    }

    public function setSuperModerator($v1, $raw = false)
    {
        $this->setFieldValue('super_moderator', $v1, $raw);
    }


    public function getLanguage()
    {
        return $this->getFieldValue('language');
    }

    public function setLanguage($v1, $raw = false)
    {
        $this->setFieldValue('language', $v1, $raw);
    }
}
