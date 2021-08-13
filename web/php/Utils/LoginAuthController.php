<?php

namespace Wikidot\Utils;


use Illuminate\Support\Facades\Cache;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Ozone;
use Ozone\Framework\RunData;
use Ozone\Framework\WebFlowController;
use Wikidot\DB\OzoneSessionPeer;

class LoginAuthController extends WebFlowController
{

    public static function getSecretSeed() {
        return GlobalProperties::$SECRET_LOGIN_SEED;
    }

    public function process()
    {

        Ozone ::init();

        $runData = new RunData();
        $runData->init();

        Ozone::setRunData($runData);

        /* Get session cookie.*/
        if(GlobalProperties::$SESSION_COOKIE_SECURE == true) {
            $sessionId = $_COOKIE[GlobalProperties::$SESSION_COOKIE_NAME_SSL];
        }
        else {
            $sessionId = $_COOKIE[GlobalProperties::$SESSION_COOKIE_NAME];
        }
            if(!$sessionId) {
                throw new ProcessException('Please accept cookies in your browser. (secure)');
            }

        $pl = $runData->getParameterList();
        $sessionHash = $pl->getParameterValue('sessionHash');

        /* Select session from the database. */
        $c = new Criteria();
        $c->add('session_id', $sessionId);
        $c->add("md5(session_id || '".self::getSecretSeed()."')", $sessionHash);

        $session = OzoneSessionPeer::instance()->selectOne($c);

        if (!$session) {
            # This appears to have broken when using HTTPS logins. They get logged in anyway.
            # Just redirect them to where they wanted to go.
            $url = $pl->getParameterValue('origUrl');
            if (!$url) {
                $url = GlobalProperties::$HTTP_SCHEMA . "://" . GlobalProperties::$URL_HOST;
            }

            //echo $url;
            header('HTTP/1.1 301 Moved Permanently');
            header("Location: $url");
            return;
        }

        /* Set IP strings. */
        /* Assume that the previous ip was obtained using the SSL proto.
           If not, this controller should not be invoked at all. */

        $session->setIpAddressSsl($runData->createIpString());
        $session->setIpAddress($runData->createIpString());

        $session->save();

        /* IMPORTANT: Also clear the session cache. */
        $key = 'session..'.$session->getSessionId();
        Cache::put($key, $session, 600);


        /* If everything went well, redirect to the original URL. */

        $url = $pl->getParameterValue('origUrl');
        if (!$url) {
            $url = GlobalProperties::$HTTP_SCHEMA . "://" . GlobalProperties::$URL_HOST;
        }

        header('HTTP/1.1 301 Moved Permanently');
        header("Location: $url");
    }
}
