<?php

namespace Wikidot\Actions;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Database\Database;
use Ozone\Framework\ODate;
use Ozone\Framework\Ozone;
use Ozone\Framework\SmartyAction;
use Wikidot\DB\OzoneUserPeer;
use Wikidot\DB\OzoneSessionPeer;
use Wikidot\Utils\EventLogger;
use Wikidot\Utils\GlobalProperties;
use Wikidot\Utils\ProcessException;

class LoginAction extends SmartyAction
{

    public function perform($r)
    {
    }

    public function loginEvent($runData)
    {
        $pl = $runData->getParameterList();
        $uname = $pl->getParameterValue("name");
        $upass = $pl->getParameterValue("password");

        $userId = $pl->getParameterValue("welcome");

        $keepLogged = $pl->getParameterValue("keepLogged");
        $bindIP = $pl->getParameterValue("bindIP");

        // decrypt! woooohhooooo!!!!!!!!

        $seed = $runData->sessionGet("login_seed");

        if ($seed == null) {
            throw new ProcessException(_("Your session has expired. Please log in again."), "no_seed");
        }

        if ($userId && is_numeric($userId) && $userId >0) {
            $user = OzoneUserPeer::instance()->selectByPrimaryKey($userId);
            if (!$user or password_verify($upass, $user->getPassword())) {
                $user = null;
                EventLogger::instance()->logFailedLogin($uname);
                throw new ProcessException(_("The login and password do not match."), "login_invalid");
            } else {
                $runData->resetSession();
                $session = $runData->getSession();
                $session->setUserId($user->getUserId());
                // set other parameters
                $session->setStarted(new ODate());
                $session->setLastAccessed(new ODate());

                $user->setLastLogin(new ODate());
                $user->save();

                if ($keepLogged) {
                    $session->setInfinite(true);
                }
                if ($bindIP) {
                    $session->setCheckIp(true);
                }

                setsecurecookie("welcome", $user->getUserId(), time() + 10000000, "/", GlobalProperties::$SESSION_COOKIE_DOMAIN);

                // log event
                EventLogger::instance()->logLogin();
            }
        } else {
            $user = null;
            EventLogger::instance()->logFailedLogin($uname);
            throw new ProcessException(_("Login failed. Please report this (LoginAction::loginEvent)"), "login_invalid");
        }
    }

    public function loginCancelEvent($runData)
    {
        $runData->sessionDel("login_seed");
    }

    public function logoutEvent($runData)
    {
        $db = Database::connection();
        $db->begin();
            EventLogger::instance()->logLogout();
        if ($runData->getUser()) {
            $userId = $runData->getUser()->getUserId();
        }

        $runData->sessionStop();

        // be even wiser! delete all sessions by this user from the current IP string!
        if ($userId !== null) {
            $c = new Criteria();
            $c->add("user_id", $userId);
            $c->add("ip_address", $runData->createIpString());
            // outdate the cache first
            $ss = OzoneSessionPeer::instance()->select($c);
            $mc = OZONE::$memcache;
            foreach ($ss as $s) {
                $mc->delete('session..'.$s->getSessionId());
            }
            OzoneSessionPeer::instance()->delete($c);
        }

        $db->commit();
    }
}
