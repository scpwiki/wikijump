<?php

namespace Wikidot\Modules\Edit;


use Wikidot\DB\PageRevisionPeer;


use Ozone\Framework\SmartyModule;
use Wikidot\Utils\Diff;

class PageEditDiffModule extends SmartyModule
{

    public function build($runData)
    {
        $pl = $runData->getParameterList();
        $toPageSource = $pl->getParameterValue("source");
        $revisionId = $pl->getParameterValue("revision_id");

        $revision = PageRevisionPeer::instance()->selectByPrimaryKey($revisionId);
        $fromPageSource = $revision->getSourceText();

        // create page diff... wooo...

        $t1 = $fromPageSource;
        $t2 = $toPageSource;

        $inlineDiff = Diff::generateInlineStringDiff($t1, $t2);
        $runData->contextAdd("diff", $inlineDiff);
    }
}
