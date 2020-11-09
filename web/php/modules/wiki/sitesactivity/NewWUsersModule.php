<?php
use DB\OzoneUserPeer;

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

        $c = new Criteria();
        $c->add('user_id', 0, '>');
        $c->addOrderDescending("user_id");

        $c->setLimit(5);

        $users = OzoneUserPeer::instance()->select($c);

        $runData->contextAdd("users", $users);
    }
}
