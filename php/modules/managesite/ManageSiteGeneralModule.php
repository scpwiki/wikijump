<?php
use DB\SiteTagPeer;

class ManageSiteGeneralModule extends ManageSiteBaseModule
{

    public function build($runData)
    {

        $site = $runData->getTemp("site");

        // get tags
        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        $dbTags = SiteTagPeer::instance()->select($c);
        $tags = '';
        foreach ($dbTags as $dbTag) {
            $tags .= htmlspecialchars($dbTag->getTag()).' ';
        }
        $tags = trim($tags);

        $runData->contextAdd("tags", $tags);
        $runData->contextAdd("site", $site);
    }
}
