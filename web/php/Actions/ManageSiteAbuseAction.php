<?php

namespace Wikidot\Actions;
use Ozone\Framework\Database\Database;
use Ozone\Framework\SmartyAction;

use Wikidot\Utils\ProcessException;
use Wikidot\Utils\WDPermissionManager;
use Wikijump\Models\User;

class ManageSiteAbuseAction extends SmartyAction
{

    public function isAllowed($runData)
    {
        WDPermissionManager::instance()->hasPermission('manage_site', $runData->getUser(), $runData->getTemp("site"));
        return true;
    }

    public function perform($r)
    {
    }

    public function clearPageFlagsEvent($runData)
    {
        $site = $runData->getTemp("site");
        $pl = $runData->getParameterList();

        $path = $pl->getParameterValue("path");

        if ($path == null || $path == '') {
            throw new ProcessException(_("Error processing the request. No page specified"), "no_path");
        }

        $q = "UPDATE page_abuse_flag SET site_valid=FALSE WHERE " .
                "site_id='".$site->getSiteId()."' " .
                "AND path='".db_escape_string($path)."' " .
                "AND site_valid=TRUE";

        $db = Database::connection();
        $db->query($q);
    }

    public function clearUserFlagsEvent($runData)
    {
        $site = $runData->getTemp("site");
        $pl = $runData->getParameterList();

        $targetUserId = $pl->getParameterValue("userId");
        $targetUser = User::find($targetUserId);

        if ($targetUser == null) {
            throw new ProcessException(_("Error processing the request. No user found."), "no_user");
        }

        $q = "UPDATE user_abuse_flag SET site_valid=FALSE WHERE " .
                "site_id='".$site->getSiteId()."' " .
                "AND target_user_id='".$targetUser->getUserId()."' " .
                "AND site_valid=TRUE";

        $db = Database::connection();
        $db->query($q);
    }

    public function clearAnonymousFlagsEvent($runData)
    {
        $site = $runData->getTemp("site");
        $pl = $runData->getParameterList();

        $address = $pl->getParameterValue("address");
        $proxy = $pl->getParameterValue("proxy") ?? null;
        if ($proxy) {
            $proxy = "TRUE";
        } else {
            $proxy = "FALSE";
        }

        if (preg_match('/^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+/', $address) !==1) {
            throw new ProcessException(_("Wrong address format."), "bad_address");
        }

        $q = "UPDATE anonymous_abuse_flag SET site_valid=FALSE WHERE " .
                "site_id='".$site->getSiteId()."' " .
                "AND address='$address' " .
                "AND proxy=$proxy ".
                "AND site_valid=TRUE";

        $db = Database::connection();
        $db->query($q);
    }
}
