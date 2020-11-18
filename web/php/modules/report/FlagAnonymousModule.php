<?php
use DB\AnonymousAbuseFlagPeer;

class FlagAnonymousModule extends SmartyModule
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

        $userString = $pl->getParameterValue("userString");
        if ($userString == null || $userString == '') {
            throw new ProcessException(_("Error processing the request."), "no_user_string");
        }

        // check if userString match the IP pattern

        if (preg_match('/^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+(\|[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+)?$/', $userString) !==1) {
            throw new ProcessException(_("Error processing the request."), "bad_user_string");
        }

        $site = $runData->getTemp("site");
        $user = $runData->getUser();

        // which to use which not.

        $ips = explode('|', $userString);

        $flagged = true;

        $valid1 = false;

        foreach ($ips as $ip) {
            // check if private
            if (false && preg_match("/^(10\..*)|(172\.16\..*)|(192\.168\..*)|(127\..*)|(169\.254\..*)/", $ip) !=0) {
                continue;
            }
            $valid1 = true;

            $c = new Criteria();
            $c->add("address", $ip);
            $c->add("user_id", $user->getUserId());

            $flag = AnonymousAbuseFlagPeer::instance()->selectOne($c);
            if ($flag) {
                $flagged = $flagged && true;
            } else {
                $flagged = false;
            }
        }

        if (!$valid1) {
            throw new ProcessException(_("IP address of the user belongs to a private subnet. Sorry, such an address cannot be flagged."));
        }

        if ($flagged) {
            $runData->contextAdd("flagged", true);
        }

        $runData->contextAdd("userString", $userString);
        list($ip, $proxy) = explode("|", $userString);
        $runData->contextAdd("ip", $ip);
        $runData->contextAdd("proxy", $proxy);
    }
}
