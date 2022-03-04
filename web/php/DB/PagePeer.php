<?php

namespace Wikidot\DB;

use Ds\Set;
use Illuminate\Support\Facades\DB;
use Ozone\Framework\Database\Criteria;
use Wikidot\Utils\WDStringUtils;use Wikijump\Services\Deepwell\DeepwellService;

/**
 * Object Model Class.
 *
 */
class PagePeer extends PagePeerBase
{
    public function selectByName(string $site_id, string $name)
    {
        $c = new Criteria();
        $c->add("site_id", $site_id);
        $c->add("unix_name", WDStringUtils::toUnixName($name));
        return $this->selectOne($c);
    }
}
