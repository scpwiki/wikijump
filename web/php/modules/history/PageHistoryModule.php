<?php
use DB\PageRevisionPeer;

class PageHistoryModule extends SmartyModule
{

    public function build($runData)
    {
        $pageId = $runData->getParameterList()->getParameterValue("page_id");

        if (!$pageId || !is_numeric($pageId)) {
            throw new ProcessException(_("The page can not be found or does not exist."), "no_page");
        }

        $c = new Criteria();
        $c->add('page_id', $pageId);
        $c->addOrderDescending('revision_id');

        $pageRevisions = PageRevisionPeer::instance()->select($c);
        $runData->contextAdd("pageRevisions", $pageRevisions);
    }
}
