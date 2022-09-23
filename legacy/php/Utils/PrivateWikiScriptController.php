<?php

namespace Wikidot\Utils;

use Ozone\Framework\Ozone;
use Ozone\Framework\RunData;

class PrivateWikiScriptController extends UploadedFileFlowController
{

    public function process()
    {

        Ozone::init();
        $runData = new RunData();
        $runData->init();
        Ozone::setRunData($runData);

        $runData->handleSessionStart();
        $user = $runData->getUser();
        $site = $this->siteFromHost($_SERVER['HTTP_HOST'], false, true);

        if (! $this->userAllowed($user, $site)) {
            $this->setContentTypeHeader("text/javascript");
            echo "window.location = '/local--auth/' + encodeURIComponent(window.location);";
        }
    }
}
