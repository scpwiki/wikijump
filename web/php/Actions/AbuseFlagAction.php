<?php

namespace Wikidot\Actions;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Database\Database;
use Ozone\Framework\SmartyAction;
use Wikidot\DB\PageAbuseFlagPeer;
use Wikidot\DB\PageAbuseFlag;

use Wikidot\DB\UserAbuseFlagPeer;
use Wikidot\DB\SitePeer;
use Wikidot\DB\UserAbuseFlag;
use Wikidot\DB\AnonymousAbuseFlagPeer;
use Wikidot\DB\AnonymousAbuseFlag;
use Wikidot\Utils\EventLogger;
use Wikidot\Utils\GlobalProperties;
use Wikidot\Utils\ProcessException;
use Wikidot\Utils\WDPermissionException;
use Wikijump\Models\User;

class AbuseFlagAction extends SmartyAction
{

    public function isAllowed($runData)
    {
        $userId = $runData->getUserId();
        if (!$userId) {
            throw new WDPermissionException(_("This option is available only to registered (and logged-in) users."));
        }
        return true;
    }

    public function perform($r)
    {
    }

    public function flagPageEvent($runData)
    {
        $pl = $runData->getParameterList();

        $path = $pl->getParameterValue("path");
        $toFlag = $pl->getParameterValue("flag") ?? null;
        if ($path == null || $path == '') {
            throw new ProcessException(_("Error processing the request."), "no_path");
        }
        $site = $runData->getTemp("site");

        $user = $runData->getUser();

        $db = Database::connection();
        $db->begin();

        if ($toFlag) {
            // flag the page

            // check if not flagged already
            $c = new Criteria();
            $c->add("user_id", $user->id);
            $c->add("site_id", $site->getSiteId());
            $c->add("path", $path);

            $flag = PageAbuseFlagPeer::instance()->selectOne($c);

            if ($flag == null) {
                $flag = new PageAbuseFlag();
                $flag->setUserId($user->id);
                $flag->setSiteId($site->getSiteId());
                $flag->setPath($path);
                $flag->save();
                EventLogger::instance()->logFlagPage($path);
            }
        } else {
            // unflag
            $c = new Criteria();
            $c->add("user_id", $user->id);
            $c->add("site_id", $site->getSiteId());
            $c->add("path", $path);
            PageAbuseFlagPeer::instance()->delete($c);
            EventLogger::instance()->logUnflagPage($path);
        }

        $db->commit();
    }

    public function flagUserEvent($runData)
    {
        $pl = $runData->getParameterList();

        $toFlag = $pl->getParameterValue("flag");

        $targetUserId = $pl->getParameterValue("targetUserId");

        if ($targetUserId == null || $targetUserId == '' || !is_numeric($targetUserId)) {
            throw new ProcessException(_("Error processing the request."), "no_target_user");
        }

        $targetUser = User::find($targetUserId);
        if ($targetUser == null) {
            throw new ProcessException(__("Error processing the request."), "no_target_user");
        }

        $site = $runData->getTemp("site");

        $user = $runData->getUser();

        $db = Database::connection();
        $db->begin();

        if ($toFlag) {
            // flag the user

            // check if not flagged already
            $c = new Criteria();
            $c->add("user_id", $user->id);
            $c->add("target_user_id", $targetUser->id);

            $flag = UserAbuseFlagPeer::instance()->selectOne($c);

            if ($flag == null) {
                $siteId = $site->getSiteId();
                // get the host if any
                $host = $pl->getParameterValue("host") ?? null;
                if ($host) {
                    if (preg_match("/^([a-zA-Z0-9\-]+)\." . GlobalProperties::$URL_DOMAIN_PREG . "$/", $host, $matches)==1) {
                        $siteUnixName=$matches[1];
                        $c = new Criteria();
                        $c->add("unix_name", $siteUnixName);
                        $siter = SitePeer::instance()->selectOne($c);
                    } else {
                        $c = new Criteria();
                        $c->add("custom_domain", $host);
                        $siter = SitePeer::instance()->selectOne($c);
                    }

                    if ($siter !== null) {
                        $siteId = $siter->getSiteId();
                    }
                }

                $flag = new UserAbuseFlag();
                $flag->setUserId($user->id);
                $flag->setSiteId($siteId);
                $flag->setTargetUserId($targetUser->id);
                $flag->save();
                EventLogger::instance()->logFlagUser($targetUser);
            }
        } else {
            // unflag
            $c = new Criteria();
            $c->add("user_id", $user->id);
            $c->add("target_user_id", $targetUser->id);
            UserAbuseFlagPeer::instance()->delete($c);
            EventLogger::instance()->logUnflagUser($targetUser);
        }

        $db->commit();
    }

    public function flagAnonymousEvent($runData)
    {
        $pl = $runData->getParameterList();

        $toFlag = $pl->getParameterValue("flag");

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

        $db = Database::connection();
        $db->begin();

        $ips = explode('|', $userString);

        if ($toFlag) {
            $i = 0;
            foreach ($ips as $ip) {
                $i++;
                if (false && preg_match("/^(10\..*)|(172\.16\..*)|(192\.168\..*)|(127\..*)|(169\.254\..*)/", $ip) !=0) {
                    # Another file had a process exception thrown here for local ip
                    continue;
                }
                // flag the IP

                // check if not flagged already
                $c = new Criteria();
                $c->add("user_id", $user->id);
                $c->add("address", $ip);

                $flag = AnonymousAbuseFlagPeer::instance()->selectOne($c);

                if ($flag == null) {
                    $siteId = $site->getSiteId();

                    $flag = new AnonymousAbuseFlag();
                    $flag->setUserId($user->id);
                    $flag->setSiteId($siteId);
                    $flag->setAddress($ip);
                    if ($i == 2) {
                        $flag->setProxy(true);
                    }
                    $flag->save();
                }
            }

            EventLogger::instance()->logFlagAnonymous($userString);
        } else {
            foreach ($ips as $ip) {
                //  unflag
                $c = new Criteria();
                $c->add("user_id", $user->id);
                $c->add("address", $ip);
                AnonymousAbuseFlagPeer::instance()->delete($c);
            }
            EventLogger::instance()->logUnflagAnonymous($userString);
        }

        $db->commit();
    }
}
