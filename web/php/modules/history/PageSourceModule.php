<?php
use DB\PageRevisionPeer;

class PageSourceModule extends SmartyModule
{
    public function build($runData)
    {
        $revisionId = $runData->getParameterList()->getParameterValue("revision_id");

        $revision = PageRevisionPeer::instance()->selectByPrimaryKey($revisionId);
        if ($revision == null) {
            throw new ProcessException(_("Revision error"), "revision_error");
        }
        $source = $revision->getSourceText();

        $runData->contextAdd("source", $source);
        $runData->contextAdd("revisionNo", $revision->getRevisionNumber());
    }
}
