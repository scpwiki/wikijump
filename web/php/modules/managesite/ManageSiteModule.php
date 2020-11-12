<?php
class ManageSiteModule extends ManageSiteBaseModule
{

    protected $processPage = true;

    public function build($runData)
    {

        $pl = $runData->getParameterList();
        $start = $pl->getParameterValue("start");
        if ($start) {
            $runData->contextAdd("start", $start);
        }

        $site = $runData->getTemp("site");

        $runData->contextAdd("site", $site);

        $runData->contextAdd('useCustomDomains', GlobalProperties::$USE_CUSTOM_DOMAINS);
        $runData->contextAdd('useSsl', GlobalProperties::$USE_SSL);
        $runData->contextAdd('allowHttp', GlobalProperties::$ALLOW_ANY_HTTP);
    }

    public function processPage($out, $runData)
    {

        $out = preg_replace("/<div id=\"page\-title\">(.*?)<\/div>/is", "", $out, 1);

        return $out;
    }
}
