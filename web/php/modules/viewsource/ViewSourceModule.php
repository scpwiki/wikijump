<?php
use DB\PagePeer;

class ViewSourceModule extends SmartyModule
{
    public function build($runData)
    {
        $site = $runData->getTemp("site");
        $pageId = $runData->getParameterList()->getParameterValue("page_id");

        $raw = $runData->getParameterList()->getParameterValue("raw");

        if (!$pageId || !is_numeric($pageId)) {
            throw new ProcessException(_("The page can not be found or does not exist."), "no_page");
        }

        $page = PagePeer::instance()->selectByPrimaryKey($pageId);

        if (!$page || $page->getSiteId() !== $site->getSiteId()) {
            throw new ProcessException(_("The page can not be found or does not exist."), "no_page");
        }

        $source = $page->getCurrentRevision()->getSourceText();

        $runData->contextAdd("source", $source);
        $runData->contextAdd("raw", $raw);
    }
}
