<?php

namespace Wikidot\DB;


use Illuminate\Support\Facades\Cache;
use Ozone\Framework\Ozone;

/**
 * Object Model Class.
 *
 */
class DomainRedirect extends DomainRedirectBase
{

    public function save()
    {
        $key = 'domain_redirect..'.$this->getUrl();
        Cache::forget($key);

        parent::save();
    }
}
