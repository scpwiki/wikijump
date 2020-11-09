<?php
use DB\ContactPeer;

class NumberContacts implements KarmaRuleInterface
{

    public function calculate($user)
    {
        $c = new Criteria();
        $c->add('user_id', $user->getUserId());
        $c->addOr('target_user_id', $user->getUserId());
        $count = ContactPeer::instance()->selectCount($c);
        return $count;
    }
}
