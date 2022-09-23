<?php

namespace Wikidot\Modules\Wiki\SitesActivity;


use Illuminate\Support\Facades\Cache;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Ozone;


use Ozone\Framework\SmartyModule;
use Wikijump\Models\User;

class NewWUsersModule extends SmartyModule
{

    public function render($runData)
    {
        $key = "module..0..NewWUsersModule";

        $out = Cache::get($key);
        if (!$out) {
            $out = parent::render($runData);
            Cache::put($key, $out, 180);
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
