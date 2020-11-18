<?php
use DB\PageAbuseFlagPeer;

class FlagPageModule extends SmartyModule
{

    public function isAllowed($runData)
    {
        $userId = $runData->getUserId();
        if ($userId == null || $userId <1) {
            throw new WDPermissionException(_("This option is available only to registered (and logged-in) users."));
        }
        return true;
    }

    public function build($runData)
    {
        $pl = $runData->getParameterList();

        $path = $pl->getParameterValue("path");
        if ($path == null || $path == '') {
            throw new ProcessException(_("Error processing the request."), "no_path");
        }
        $site = $runData->getTemp("site");
        $user = $runData->getUser();
        // check if flagged already
        $c = new Criteria();
        $c->add("user_id", $user->getUserId());
        $c->add("site_id", $site->getSiteId());
        $c->add("path", $path);

        $flag = PageAbuseFlagPeer::instance()->selectOne($c);

        if ($flag) {
            $runData->contextAdd("flagged", true);
        }
    }
}
