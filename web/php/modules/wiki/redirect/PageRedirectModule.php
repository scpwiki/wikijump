<?php
class PageRedirectModule extends SmartyModule
{

    public function build($runData)
    {
        $pl = $runData->getParameterList();

        $noRedirect = (bool) $pl->getParameterValue("noredirect");

        if ($runData->isAjaxMode()) {
            $noRedirect = true;
        }

        $target = trim($pl->getParameterValue("destination"));

        if ($target == "") {
            throw new ProcessException(_('No redirection destination specified. Please use the destination="page-name" or destination="url" attribute.'));
        }

        $currentUri = $_SERVER['REQUEST_URI'];

        if (!$noRedirect) {
            // ok, redirect!!!

            // check if mapping should be done.
            if ($target[strlen($target)-1] === '/' && strpos($currentUri, '/', 1)) {
                $map = true;
            } else {
                $map = false;
            }

            // check if $target is an URI or just a page name
            if (!strpos($target, '://')) {
                $target = WDStringUtils::toUnixName($target);
                $target = '/'.$target;
                if ($map) {
                    $target .= '/';
                }
            }

            if ($map) {
                // use more advanced mapping

                //strip page name and take the remaining part
                $mappedUri = substr($currentUri, strpos($currentUri, '/', 1)+1);
                $target .= $mappedUri;
            }

            header('HTTP/1.1 301 Moved Permanently');
            header('Location: '.$target);
            exit();
        } else {
            $runData->contextAdd("target", $target);
        }
    }
}
