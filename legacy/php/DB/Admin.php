<?php

namespace Wikidot\DB;


use Wikijump\Models\User;

/**
 * Object Model mapped Class.
 *
 */
class Admin extends AdminBase
{

    public function getSite()
    {
        if (is_array($this->prefetched)) {
            if (in_array('site', $this->prefetched)) {
                if (in_array('site', $this->prefetchedObjects)) {
                    return $this->prefetchedObjects['site'];
                } else {
                    $obj = new Site($this->sourceRow);
                    $obj->setNew(false);
                    $this->prefetchedObjects['site'] = $obj;
                    return $obj;
                }
            }
        }
        return SitePeer::instance()->selectByPrimaryKey($this->getSiteId());
    }

    public function getUser()
    {
        return User::find($this->getUserId());
    }
}
