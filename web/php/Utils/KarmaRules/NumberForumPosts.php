<?php

namespace Wikidot\Utils\KarmaRules;

use Ozone\Framework\Database\Criteria;
use Wikidot\DB\ForumPostPeer;
use Wikidot\Utils\KarmaRuleInterface;

class NumberForumPosts implements KarmaRuleInterface
{

    public function calculate($user)
    {
        $c = new Criteria();
        $c->add('user_id', $user->id);
        $count = ForumPostPeer::instance()->selectCount($c);
        return $count;
    }
}
