<?php

namespace Wikidot\DB;


use Ozone\Framework\Ozone;

/**
 * Object Model Class.
 *
 */
class DomainRedirect extends DomainRedirectBase
{

    public function save()
    {
        $memcache = Ozone::$memcache;
        $key = 'domain_redirect..'.$this->getUrl();
        $memcache->delete($key);

        parent::save();
    }
}
