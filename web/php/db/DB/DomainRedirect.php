<?php
namespace DB;

/**
 * Object Model class.
 *
 */
class DomainRedirect extends DomainRedirectBase
{

    public function save()
    {
        $memcache = \Ozone::$memcache;
        $key = 'domain_redirect..'.$this->getUrl();
        $memcache->delete($key);

        parent::save();
    }
}
