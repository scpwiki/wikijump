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
        $mode = $pl->getParameterValue("mode");
        $revisionId = $pl->getParameterValue("revision_id");

        $revision = PageRevisionPeer::instance()->selectByPrimaryKey($revisionId);
        $fromPageSource = $revision->getSourceText();

        if ($mode == "section") {
            // compare only a fragment...
            $rangeStart = $pl->getParameterValue("range_start");
            $rangeEnd = $pl->getParameterValue("range_end");

            $s2 = explode("\n", $fromPageSource);
            $fromPageSource = implode("\n", array_slice($s2, $rangeStart, $rangeEnd-$rangeStart+1));
        }

        // create page diff... wooo...

        $t1 = $fromPageSource;
        $t2 = $toPageSource;

        $inlineDiff = Diff::generateInlineStringDiff($t1, $t2);
        $runData->contextAdd("diff", $inlineDiff);
    }
}
