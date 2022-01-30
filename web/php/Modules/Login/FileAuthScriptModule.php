<?php

namespace Wikidot\Modules\Login;

use Ozone\Framework\SmartyModule;
use Wikidot\Utils\GlobalProperties;
use Wikidot\Utils\UploadedFileFlowController;

class FileAuthScriptModule extends SmartyModule
{

    public function build($runData)
    {
        $site = $runData->getTemp('site');
        // TODO: ControllerUtils
        $u = new UploadedFileFlowController();
        if ($runData->getUser() && $site->getPrivate() && $u->userAllowed($runData->getUser(), $site)) {
            $pwdomain = $site->getSlug() . "." . GlobalProperties::$URL_UPLOAD_DOMAIN;
            $pwproto = ($_SERVER["HTTPS"]) ? "https" : "http";
            $pwurl = "$pwproto://$pwdomain/filesauth.php";

            $runData->contextAdd("usePrivateWikiScript", true);
            $runData->contextAdd("privateWikiScriptUrl", $pwurl);
        }
    }
}
