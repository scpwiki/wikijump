<?php

namespace Wikidot\Actions;
use Illuminate\Support\Facades\Auth;
use Illuminate\Support\Facades\Cache;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Database\Database;
use Ozone\Framework\ODate;
use Ozone\Framework\Ozone;
use Ozone\Framework\SmartyAction;

use Wikidot\DB\OzoneSessionPeer;
use Wikidot\Utils\EventLogger;
use Wikidot\Utils\GlobalProperties;
use Wikidot\Utils\ProcessException;
use Wikijump\Models\User;

class LoginAction extends SmartyAction
{

    public function perform($r)
    {
    }

    public function loginEvent($runData)
    {
        $pl = $runData->getParameterList();
        $uname = strtolower($pl->getParameterValue("name"));
        $upass = $pl->getParameterValue("password");

        $userId = $pl->getParameterValue("welcome");

        $keepLogged = $pl->getParameterValue("keepLogged");
        $bindIP = $pl->getParameterValue("bindIP");

        // decrypt! woooohhooooo!!!!!!!!

        if ($runData->sessionGet("login_seed") == null) {
            throw new ProcessException(_("Your session has expired. Please log in again."), "no_seed");
        }

        if (
            $uname != null
            && strtolower($uname) != 'automatic'
            && strtolower($uname) != 'anonymous'
        ) {
            $attempt = Auth::attempt([
                'username' => $uname,
                'password' => $upass
            ]);
            if ($attempt == false) {
                $user = null;
                EventLogger::instance()->logFailedLogin($uname);
                throw new ProcessException(_("The login and password do not match."), "login_invalid");
            } else {
                $user = User::whereRaw('lower(username)', $uname)->first();
                $runData->resetSession();
                $session = $runData->getSession();
                $session->setUserId($user->id);
                // set other parameters
                $session->setStarted(new ODate());
                $session->setLastAccessed(new ODate());

                $user->touch();
                $user->save();

                if ($keepLogged) {
                    $session->setInfinite(true);
                }
                if ($bindIP) {
                    $session->setCheckIp(true);
                }

                Auth::login($user, $keepLogged);

                setsecurecookie("welcome", $user->id, time() + 10000000, "/", GlobalProperties::$SESSION_COOKIE_DOMAIN);

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
            $userId = $runData->getUser()->id;
        }

        $runData->sessionStop();

        // be even wiser! delete all sessions by this user from the current IP string!
        if ($userId !== null) {
            $c = new Criteria();
            $c->add("user_id", $userId);
            $c->add("ip_address", $runData->createIpString());
            // outdate the cache first
            $ss = OzoneSessionPeer::instance()->select($c);
            foreach ($ss as $s) {
                Cache::forget('session..'.$s->getSessionId());
            }
            OzoneSessionPeer::instance()->delete($c);
        }

        $db->commit();
    }
}
