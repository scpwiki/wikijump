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
    }
}
