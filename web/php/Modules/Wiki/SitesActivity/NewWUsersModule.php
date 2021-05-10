<?php

namespace Wikidot\Modules\Wiki\SitesActivity;


use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Ozone;


use Ozone\Framework\SmartyModule;
use Wikijump\Models\User;

class NewWUsersModule extends SmartyModule
{

    public function render($runData)
    {
        $key = "module..0..NewWUsersModule";
        $mc = OZONE::$memcache;

        $out = $mc->get($key);
        if (!$out) {
            $out = parent::render($runData);
            $mc->set($key, $out, 0, 180);
        }

        return $out;
    }

    public function build($runData)
    {
        // get a few new users
        $users = User::latest()->limit(5);

        $runData->contextAdd("users", $users);
    }
}
