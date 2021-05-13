<?php

namespace Wikidot\DB;

use Ozone\Framework\Database\Criteria;
use Wikidot\Utils\WDStringUtils;

/**
 * Object Model Class.
 *
 */
class PagePeer extends PagePeerBase
{

    public function selectByName($siteId, $name)
    {
        $c = new Criteria();
        $c->add("site_id", $siteId);
        $c->add("unix_name", WDStringUtils::toUnixName($name));
        return $this->selectOne($c);
    }
}
