<?php
use DB\CategoryPeer;

class SiteChangesModule extends CacheableModule
{

    protected $timeOut = 60;

    protected $processPage = true;

    public function build($runData)
    {

        $site = $runData->getTemp("site");

        // get all categories
        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        $c->addOrderAscending("replace(name, '_', '00000000')");

        $categories = CategoryPeer::instance()->select($c);

        $runData->contextAdd("categories", $categories);
    }

    public function processPage($out, $runData)
    {

        $site = $runData->getTemp("site");
        $out = preg_replace(
            "/<\/head>/",
            '<link rel="alternate" type="application/rss+xml" title="Recent changes from '.htmlspecialchars($site->getName()).'" href="/feed/site-changes.xml"/></head>',
            $out,
            1
        );

        return $out;
    }
}
