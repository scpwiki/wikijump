<?php

namespace Wikidot\Modules\Wiki\Redirect;

use Ozone\Framework\SmartyModule;
use Wikidot\Utils\ProcessException;
use Wikidot\Utils\WDStringUtils;
use Wikijump\Helpers\LegacyTools;

class PageRedirectModule extends SmartyModule
{

    public function build($runData)
    {
        $pl = $runData->getParameterList();
        $redirect = !$pl->getParameterValueBoolean("noredirect");

        if ($runData->isAjaxMode()) {
            $redirect = false;
        }

        $target = trim($pl->getParameterValue("destination"));

        if ($target === '') {
            throw new ProcessException(_('No redirection destination specified. Please use the destination="page-name" or destination="url" attribute.'));
        }

        $currentUri = $_SERVER['REQUEST_URI'];

        if ($redirect) {
            // Check if $target is an URI or just a page name
            if (!strpos($target, '://')) {
                $target = WDStringUtils::toUnixName($target);
                $target = '/' . $target . LegacyTools::getPageParameters();
            }

            header('HTTP/1.1 301 Moved Permanently');
            header('Location: '.$target);
            exit();
        } else {
            $runData->contextAdd("target", $target);
        }
    }
}
