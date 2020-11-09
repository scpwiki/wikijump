<?php
use DB\OzoneUserPeer;

class LoginModule3 extends SmartyModule
{

    public function build($runData)
    {
        $pl = $runData->getParameterList();

        $backUrl = $pl->getParameterValue('backUrl');
        $runData->contextAdd('backUrl', $backUrl);

        // check if reset remebered user
        $pl = $runData->getParameterList();

        if ($pl->getParameterValue("reset")) {
            setsecurecookie('welcome', 'dummy', time() - 10000000, "/", GlobalProperties::$SESSION_COOKIE_DOMAIN);
        } else {
            // check if a recognized user

            $userId = $_COOKIE['welcome'];
            if ($userId && is_numeric($userId) && $userId >0) {
                $user = OzoneUserPeer::instance()->selectByPrimaryKey($userId);
            }
            if ($user == null) {
                setsecurecookie('welcome', 'dummy', time() - 10000000, "/", GlobalProperties::$SESSION_COOKIE_DOMAIN);
            }
        }

        $runData->contextAdd("user", $user);
    }
}
