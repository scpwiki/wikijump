<?php

namespace Wikidot\DB;


use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Ozone;

/**
 * Object Model Class.
 *
 */
class OzoneUser extends OzoneUserBase
{

    public function getProfile()
    {
        if (is_array($this->prefetched)) {
            if (in_array('profile', $this->prefetched)) {
                if (in_array('profile', $this->prefetchedObjects)) {
                    return $this->prefetchedObjects['profile'];
                } else {
                    $obj = new Profile($this->sourceRow);
                    $obj->setNew(false);
                    $this->prefetchedObjects['profile'] = $obj;
                    return $obj;
                }
            }
        }
        return ProfilePeer::instance()->selectByPrimaryKey($this->getUserId());
    }

    public function getSettings()
    {
        return UserSettingsPeer::instance()->selectByPrimaryKey($this->getUserId());
    }

    public function getKarmaLevel()
    {
        $c = new Criteria();
        $c->add('user_id', $this->getUserId());
        $karma = UserKarmaPeer::instance()->selectOne($c);
        if ($karma) {
            return $karma->getLevel();
        } else {
            return 0;
        }
    }

    public function save()
    {
        $memcache = Ozone::$memcache;
        $key = 'user..' . $this->getUserId();
        $memcache->delete($key);
        parent::save();
    }
}
