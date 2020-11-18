<?php
namespace DB;

/**
 * Object Model class.
 *
 */
class OzoneUserPeer extends OzoneUserPeerBase
{

    public function selectByPrimaryKeyCached($userId)
    {
        $memcache = \Ozone::$memcache;
        $key = 'user..' . $userId;
        $u = $memcache->get($key);
        if ($u != false) {
            return $u;
        } else {
            $u = $this->selectByPrimaryKey($userId);
            $memcache->set($key, $u, 0, 864000);
            return $u;
        }
    }
}
