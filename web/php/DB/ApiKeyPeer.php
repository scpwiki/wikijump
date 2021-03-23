<?php

namespace Wikidot\DB;


//please extend this Class
class ApiKeyPeer extends ApiKeyPeerBase
{
    public static function getUserByKey($key)
    {
        $user = null;
        $api_key = self::instance()->selectByPrimaryKey($key);
        if ($api_key) {
            $user = OzoneUserPeer::instance()->selectByPrimaryKey($api_key->getUserId());
        }
        return $user;
    }
}
