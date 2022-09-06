<?php

namespace Wikidot\Modules\Users;

use Ozone\Framework\Database\Criteria;


/**
 * This Class searches for users given the query string and results in
 * an array of matches.
 */
use Ozone\Framework\SmartyModule;
use Wikijump\Models\User;

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
        $users = User::whereIn('username',$q)->limit(101)->get();

        $runData->contextAdd("users", $users);

        // also prepare an array of user_id and nickname
        $runData->ajaxResponseAdd("count", count($users));
        if (count($users) == 101) {
            $runData->ajaxResponseAdd("over100", true);
        } else {
            $runData->ajaxResponseAdd("over100", false);
        }

        $userIds = [];
        $userNames = [];
        foreach ($users as $u) {
            $userIds[] = $u->id;
            $userNames[$u->id] = htmlspecialchars($u->username);
        }
        $runData->ajaxResponseAdd("userIds", $userIds);
        $runData->ajaxResponseAdd("userNames", $userNames);
    }
}
