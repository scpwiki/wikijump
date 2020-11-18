<?php
use DB\SitePeer;

class SitesListByTagModule extends CacheableModule
{

    protected $timeOut=300;

    public function build($runData)
    {
        $pl = $runData->getParameterList();

        $tag = $pl->getParameterValue("tag");
        if ($tag === null) {
            $runData->setModuleTemplate("Empty");
            return ;
        }

        $lang = $pl->getParameterValue("lang");

        if ($lang && $lang !== "pl" && $lang !== "en") {
            $lang = null;
        }

        // get sites

        $title = $pl->getParameterValue("title");
        $runData->contextAdd("title", $title);

        $c = new Criteria();
        $c->setExplicitFrom("site, site_tag");
        $c->add("site_tag.tag", $tag);
        $c->add("site.visible", true);
        $c->add("site.private", false);
        $c->add("site.deleted", false);
        if ($lang) {
            $c->add("site.language", $lang);
        }
        $c->add("site_tag.site_id", "site.site_id", "=", false);
        $c->addOrderAscending('site.name');

        $sites = SitePeer::instance()->select($c);

    //  $q = "SELECT site.* FROM site, tag WHERE tag.tag = '".db_escape_string($tag")."'

        $runData->contextAdd("tag", $tag);
        $runData->contextAdd("sites", $sites);
        $runData->contextAdd("sitesCount", count($sites));
    }
}
