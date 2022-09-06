<?php

namespace Wikidot\Modules\Changes;

use Wikidot\Utils\CacheableModule;
use Wikijump\Services\Deepwell\Models\Category;

class SiteChangesModule extends CacheableModule
{

    protected $timeOut = 60;

    protected $processPage = true;

    public function build($runData)
    {
        $site = $runData->getTemp("site");
        $categories = Category::findAll($site->getSiteId());
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
