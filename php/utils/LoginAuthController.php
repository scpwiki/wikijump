<?php
/**
 * Wikidot - free wiki collaboration software
 * Copyright (c) 2008-2020, Wikidot Inc., SCP Wiki Technical Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * For more information about licensing visit:
 * http://www.wikidot.org/license
 *
 * @category Wikidot
 * @package Wikidot
 * @version $Id: UploadedFileFlowController.php,v 1.5 2008/08/01 14:00:27 quake Exp $
 * @copyright Copyright (c) 2008-2020, Wikidot Inc., SCP Wiki Technical Team
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */


use DB\OzoneSessionPeer;

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
        $mc = OZONE::$memcache;
        $key = 'session..'.$session->getSessionId();
        $mc->set($key, $session, 0, 600);


        /* If everything went well, redirect to the original URL. */

        $url = $pl->getParameterValue('origUrl');
        if (!$url) {
            $url = GlobalProperties::$HTTP_SCHEMA . "://" . GlobalProperties::$URL_HOST;
        }

        //echo $url;
        header('HTTP/1.1 301 Moved Permanently');
        header("Location: $url");
    }
}
