<?php

namespace Wikidot\Actions;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Database\Database;
use Ozone\Framework\ODate;
use Ozone\Framework\SmartyAction;

use Wikidot\DB\MemberPeer;
use Wikidot\DB\UserBlockPeer;
use Wikidot\DB\UserBlock;
use Wikidot\DB\IpBlockPeer;
use Wikidot\DB\IpBlock;
use Wikidot\Utils\WDPermissionManager;
use Wikijump\Models\User;

class ManageSiteBlockAction extends SmartyAction
{

    public function isAllowed($runData)
    {
        WDPermissionManager::instance()->hasPermission('manage_site', $runData->getUser(), $runData->getTemp("site"));
        return true;
    }

    public function perform($r)
    {
    }

    public function blockUserEvent($runData)
    {
        $pl = $runData->getParameterList();
        $site = $runData->getTemp("site");

        $userId = $pl->getParameterValue("userId");
        $user = User::find($userId);
        if ($user == null) {
            $runData->ajaxResponseAdd("status", "no_user");
            $runData->ajaxResponseAdd("message", "No such user.");
            return;
        }
        // check if user is a member or so.
        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        $c->add("user_id", $userId);
        $mem = MemberPeer::instance()->selectOne($c) ?? null;
        if ($mem) {
            $runData->ajaxResponseAdd("status", "user_member");
            $runData->ajaxResponseAdd("message", _("The user you want to block is a member of this site. Please first remove them from the site members list."));
            return;
        }

        // check if not already blocked
        $bl = UserBlockPeer::instance()->selectOne($c);
        if ($bl) {
            $runData->ajaxResponseAdd("status", "already_blocked");
            $runData->ajaxResponseAdd("message", _("This user is already blocked."));
            return;
        }

        // ok, now block!

        $db = Database::connection();
        $db->begin();

        $reason = $pl->getParameterValue("reason", "AMODULE");

        $block = new UserBlock();
        $block->setSiteId($site->getSiteId());
        $block->setUserId($userId);
        $block->setDateBlocked(new ODate());

        if ($reason && $reason !== '') {
            $block->setReason(substr($reason, 0, 500));
        }
        $block->save();

        $db->commit();
    }

    public function deleteBlockEvent($runData)
    {
        $pl = $runData->getParameterList();
        $site = $runData->getTemp("site");

        $userId = $pl->getParameterValue("userId");
        $user = User::find($userId);
        if ($user == null) {
            $runData->ajaxResponseAdd("status", "no_user");
            $runData->ajaxResponseAdd("message", _("No such user."));
            return;
        }
        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        $c->add("user_id", $userId);

        $block = UserBlockPeer::instance()->selectOne($c);
        if ($block == null) {
            $runData->ajaxResponseAdd("status", "no_user");
            $runData->ajaxResponseAdd("message", _("No such block."));
            return;
        }

        // ok, remove
        $db = Database::connection();
        $db->begin();

        UserBlockPeer::instance()->delete($c);

        $db->commit();
    }

    public function blockIpEvent($runData)
    {
        $pl = $runData->getParameterList();
        $site = $runData->getTemp("site");

        $ips = trim($pl->getParameterValue("ips", "AMODULE"));

        // now just parse ips and check ALL IPs if they match the pattern
        $ips = explode("\n", $ips);

        $errorIps = array(); // errors
        $ips2 = array(); // formatted list of ips

        $db = Database::connection();
        $db->begin();

        foreach ($ips as $ip) {
            // check if not private
            if (preg_match("/^(10\..*)|(172\.16\..*)|(192\.168\..*)|(127\..*)|(169\.254\..*)/", $ip) !=0) {
                $errorIps[$ip] = "Blocking from private ranges is not allowed";
                continue;
            }
            $ip2 = $ip;

            $ip2 = preg_replace("/(\.[0-9]{1,3})\/8/", ".0.0.0/8", $ip2);
            $ip2 = preg_replace("/(\.[0-9]{1,3}){2}\/16/", ".0.0/16", $ip2);
            $ip2 = preg_replace("/(\.[0-9]{1,3}){1}\/24/", ".0/24", $ip2);

            $ip2 = preg_replace("/(\.\*){3}$/", ".0.0.0/8", $ip2);
            $ip2 = preg_replace("/(\.\*){2}$/", ".0.0/16", $ip2);
            $ip2 = preg_replace("/(\.\*){1}$/", ".0/24", $ip2);

            // one more fix

            // now validate.

            if (preg_match("/^[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}(\/[0-9]{1,2})?$/", $ip2) == 0) {
                $errorIps[$ip] = _("Not a valid IP address");
                continue;
            }

            $z = explode("/", $ip2);
            $z1 = explode(".", $z[0]);
            foreach ($z1 as $zz) {
                if ($zz<0||$zz>255) {
                    $errorIps[$ip] = _("Not a valid IP address");
                    continue 2;
                }
            }
            if ($z[1] &&($z[1]<8 || $z[1]>32)) {
                $errorIps[$ip] = _("Not a valid IP address");
                continue;
            }
            // check if not in the database
            $c = new Criteria();
            $c->add("site_id", $site->getSiteId());
            $c->add("ip", $ip2);
            $bl = IpBlockPeer::instance()->selectOne($c);
            if ($bl != null) {
                $errorIps[$ip] = _("IP already in the list.");
                continue;
            }
            $ips2[] = $ip2;
        }
        if (count($errorIps)>0) {
            // create ip error messages

            $m = array();
            foreach ($errorIps as $ip => $er) {
                $m[] = "$ip - $er";
            }
            $m = implode("<br/>", $m);

            $runData->ajaxResponseAdd("errorIps", $errorIps);
            $runData->ajaxResponseAdd("errormess", $m);
            $runData->ajaxResponseAdd("status", "ip_errors");
            $runData->ajaxResponseAdd("message", _("Some rules are not valid. Nothing has been saved yet."));
        } else {
            // save!!!
            $reason = $pl->getParameterValue("reason", "AMODULE");

            // remove duplicate entries
            $ips2 =  array_unique($ips2);

            foreach ($ips2 as $ip) {
                $now = new ODate();

                $block = new IpBlock();
                $block->setSiteId($site->getSiteId());
                $block->setIp($ip);
                $block->setDateBlocked($now);
                if ($reason && $reason !== '') {
                    $block->setReason(substr($reason, 0, 500));
                }
                $block->save();
            }
        }
        $db->commit();
    }

    public function deleteIpBlockEvent($runData)
    {
        $pl = $runData->getParameterList();
        $site = $runData->getTemp("site");

        $blockId = $pl->getParameterValue("blockId");
        $block = IpBlockPeer::instance()->selectByPrimaryKey($blockId);
        if ($block == null) {
            $runData->ajaxResponseAdd("status", "no_block");
            $runData->ajaxResponseAdd("message", _("No such IP block."));
            return;
        }
        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        $c->add("block_id", $blockId);

        // ok, remove
        $db = Database::connection();
        $db->begin();

        IpBlockPeer::instance()->delete($c);

        $db->commit();
    }
}
