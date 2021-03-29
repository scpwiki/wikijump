<?php

namespace Wikidot\Utils;

use GuzzleHttp\Client;
use Wikidot\Utils\GlobalProperties;

class FriendlyCaptchaHandler
{
    public static function verifySolution($solution) {
        $client = new Client();

        // Send POST request
        $response = $client->request(
            'POST',
            'https://friendlycaptcha.com/api/v1/siteverify',
            [
                'json' => [
                    'solution' => $solution,
                    'secret' => GlobalProperties::$FR_CAPTCHA_API_KEY,
                    'sitekey' => GlobalProperties::$FR_CAPTCHA_SITE_KEY,
                ],
            ],
        );

        if ($response->getStatusCode() != 200) {
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
            return true;
        }

        $body = $response->getBody();
        $result = json_decode($body);

        // The JSON response looks like the following:
        //
        // {
        //     "success": false,
        //     "errorCodes": [
        //         "invalid_solution",
        //         "timeout_or_duplicate"
        //     ]
        // }
        //
        // The "errorCodes" field may be absent if "success" is true.

        if (!$result->success) {
            // TODO log $result->errorCodes
        }

        return $result->success;
    }
}
