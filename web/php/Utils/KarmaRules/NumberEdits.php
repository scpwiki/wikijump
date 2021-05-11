<?php

namespace Wikidot\Utils\KarmaRules;

use Ozone\Framework\Database\Criteria;
use Wikidot\DB\PageRevisionPeer;
use Wikidot\Utils\KarmaRuleInterface;

class NumberEdits implements KarmaRuleInterface
{

    public function calculate($user)
    {
        $c = new Criteria();
        $c->add('user_id', $user->id);
        $count = PageRevisionPeer::instance()->selectCount($c);
        return $count;
    }
}
