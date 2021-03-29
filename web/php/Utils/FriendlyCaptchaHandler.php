<?php

namespace Wikidot\Utils;

use Wikidot\Utils\GlobalProperties;

class FriendlyCaptchaHandler
{
    public static function verifySolution($solution) {
        $request = array(
            'solution' => $solution,
            'secret' => GlobalProperties::$FR_CAPTCHA_API_KEY,
            'sitekey' => GlobalProperties::$FR_CAPTCHA_SITE_KEY,
        );

        // TODO(aismallard): verify result with FriendlyCaptcha
        // see https://docs.friendlycaptcha.com/#/installation?id=_3-verifying-the-captcha-solution-on-the-server

        // TODO have this actually POST lol
        $response = doPostSomehow(
            'https://friendlycaptcha.com/api/v1/siteverify',
            $request,
        );

        $response = array(
            'success' => true,
            'errorCodes' => ['invalid_solution'],
        );

        if (!$response->success) {
            // TODO: log $response->errorCodes
        }

        if ($responseCode != 200) {
            // FriendlyCaptcha will always send 200 OK if the request was properly formed,
            // regardless of whether
            //
            // If another code (e.g. 401) is received, it indicates an error on our end or
            // FriendlyCaptcha's end. In such a case it is recommended we be permissive and
            // let them proceed anyways, otherwise we're breaking a core site feature until
            // the outage is resolved.
            //
            // See https://docs.friendlycaptcha.com/#/verification_api

            // TODO: log big bad error thing
        }

        return $response->success;
    }
}
