<?php
use DB\OzoneUserPeer;

/**
 * This class searches for users given the query string and results in
 * an array of matches.
 */
class UserSearchModule extends SmartyModule
{

    public function build($runData)
    {
        $query = $runData->getParameterList()->getParameterValue("query");
        // split the query by ' '
        $q = explode(' ', $query);
        // escape regex syntax now
        for ($i=0; $i<count($q); $i++) {
            $q[$i] = preg_quote($q[$i], '/');
        }
        $c = new Criteria();
        foreach ($q as $q1) {
            $c->add("nick_name", $q1, "~*");
        }
        $c->setLimit(101);

        $users = OzoneUserPeer::instance()->select($c);

        $runData->contextAdd("users", $users);

        // also prepare an array of user_id and nickname
        $runData->ajaxResponseAdd("count", count($users));
        if (count($users) == 101) {
            $runData->ajaxResponseAdd("over100", true);
        } else {
            $runData->ajaxResponseAdd("over100", false);
        }

        $userIds = array();
        $userNames = array();
        foreach ($users as $u) {
            $userIds[] = $u->getUserId();
            $userNames[$u->getUserId()] = htmlspecialchars($u->getNickName());
        }
        $runData->ajaxResponseAdd("userIds", $userIds);
        $runData->ajaxResponseAdd("userNames", $userNames);
    }
}
