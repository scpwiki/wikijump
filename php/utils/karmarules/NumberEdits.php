<?php
use DB\PageRevisionPeer;

class NumberEdits implements KarmaRuleInterface
{

    public function calculate($user)
    {
        $c = new Criteria();
        $c->add('user_id', $user->getUserId());
        $count = PageRevisionPeer::instance()->selectCount($c);
        return $count;
    }
}
