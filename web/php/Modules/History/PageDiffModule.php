<?php

namespace Wikidot\Modules\History;

use Wikidot\DB\PageRevisionPeer;
use Wikidot\DB\PagePeer;
use Wikidot\Utils\Diff;
use Wikidot\Utils\ProcessException;

use Ozone\Framework\SmartyModule;

class PageDiffModule extends SmartyModule
{

    public function build($runData)
    {
        $pl = $runData->getParameterList();

        $fromRevisionId = $pl->getParameterValue("from_revision_id");
        $toRevisionId  = $pl->getParameterValue("to_revision_id");

        if ($fromRevisionId == $toRevisionId) {
            throw new ProcessException(_("Please choose different revisions of the page to compare."), "same_revision");
        }

        $fromRevision = PageRevisionPeer::instance()->selectByPrimaryKey($fromRevisionId);
        $toRevision = PageRevisionPeer::instance()->selectByPrimaryKey($toRevisionId);

        if ($fromRevision == null || $toRevision == null) {
            throw new ProcessException(_("Error selecting revisions to compare."), "no_revisions");
        }

        $fromMetadata = $fromRevision->getMetadata();
        $toMetadata = $toRevision->getMetadata();

        $changed = array();

        // compare titles and other things
        if ($fromMetadata->getTitle() !== $toMetadata->getTitle()) {
            $changed['title'] = true;
        }
        if ($fromMetadata->getUnixName() !== $toMetadata->getUnixName()) {
            $changed['unix_name'] = true;
        }
        if ($fromMetadata->getParentPageId() !== $toMetadata->getParentPageId()) {
            $changed['parent'] = true;
            if ($fromMetadata->getParentPageId()) {
                $fromParent = PagePeer::instance()->selectByPrimaryKey($fromMetadata->getParentPageId())->getUnixName();
                $runData->contextAdd("fromParent", $fromParent);
            }
            if ($toMetadata->getParentPageId()) {
                $toParent = PagePeer::instance()->selectByPrimaryKey($toMetadata->getParentPageId())->getUnixName();
                $runData->contextAdd("toParent", $toParent);
            }
        }

        //compare source now

        $fromPageSource = $fromRevision->getSourceText();
        $toPageSource = $toRevision->getSourceText();

        if ($fromPageSource !== $toPageSource) {
            $changed['source'] = true;

            // create page diff... wooo...

            $t1 = $fromPageSource;
            $t2 = $toPageSource;

            $inlineDiff = Diff::generateInlineStringDiff($t1, $t2);
            $runData->contextAdd("inlineDiff", $inlineDiff);
        }
        $runData->contextAdd("fromPageSource", $fromPageSource);
        $runData->contextAdd("toPageSource", $toPageSource);

        $runData->contextAdd("fromRevision", $fromRevision);
        $runData->contextAdd("toRevision", $toRevision);
        $runData->contextAdd("fromMetadata", $fromMetadata);
        $runData->contextAdd("toMetadata", $toMetadata);

        $runData->contextAdd("changed", $changed);
    }
}
