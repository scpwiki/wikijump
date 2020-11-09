<?php
use DB\ForumPostPeer;

class NumberForumPosts implements KarmaRuleInterface
{

    public function calculate($user)
    {
        $c = new Criteria();
        $c->add('user_id', $user->getUserId());
        $count = ForumPostPeer::instance()->selectCount($c);
        return $count;
    }
}
