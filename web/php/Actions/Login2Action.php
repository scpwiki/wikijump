<?php

namespace Wikidot\Actions;
use Illuminate\Support\Facades\Cache;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Database\Database;
use Ozone\Framework\ODate;
use Ozone\Framework\Ozone;
use Ozone\Framework\SecurityManager;
use Ozone\Framework\SmartyAction;

use Wikidot\DB\OzoneSessionPeer;
use Wikidot\Utils\EventLogger;
use Wikidot\Utils\GlobalProperties;
use Wikidot\Utils\LoginAuthController;
use Wikidot\Utils\ProcessException;

class Login2Action extends SmartyAction
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

        // Auth via username or email.
        $sm = new SecurityManager();
        $user = $sm->authenticateUser($uname, $upass);
        if (!$user) {
            EventLogger::instance()->logFailedLogin($uname);
            throw new ProcessException(_("The login and password do not match."), "login_invalid");
        }

        $originalUrl = $runData->sessionGet('loginOriginalUrl');

        $runData->resetSession();
        $session = $runData->getSession();
        $session->setUserId($user->id);
        // set other parameters
        $session->setStarted(new ODate());
        $session->setLastAccessed(new ODate());

        /**
         * Refresh the `updated_at` value. We'll probably want a last_seen field
         * stored in the caching layer as well.
         * TODO: Add last_seen keys to memcached.
         */
        $user->touch();

        if ($keepLogged) {
            $session->setInfinite(true);
        }
        if ($bindIP) {
            $session->setCheckIp(true);
        }


            /* If the request is over https:, we should also use loginauth.php script to set non-ssl ip address. */

        if ($_SERVER['HTTPS']) {
            $sessionHash = md5($session->getSessionId() . LoginAuthController::getSecretSeed());
            $parms = array('sessionHash' => $sessionHash);
            if ($originalUrl) {
                $parms['origUrl'] = $originalUrl;
            }
            $originalUrl = GlobalProperties::$HTTP_SCHEMA . "://" . GlobalProperties::$URL_HOST . '/loginauth.php?' . http_build_query($parms);
        }

        if ($originalUrl) {
            $runData->ajaxResponseAdd('originalUrl', $originalUrl);
        }

            setsecurecookie("welcome", $user->id, time() + 10000000, "/", GlobalProperties::$SESSION_COOKIE_DOMAIN);
            setsecurecookie(GlobalProperties::$SESSION_COOKIE_NAME_IE, $runData->getSessionId(), time() + 10000000, "/", GlobalProperties::$SESSION_COOKIE_DOMAIN);

            // log event
            EventLogger::instance()->logLogin();
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
