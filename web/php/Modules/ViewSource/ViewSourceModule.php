<?php

namespace Wikidot\Modules\ViewSource;

use Wikidot\DB\PagePeer;

use Ozone\Framework\SmartyModule;
use Wikidot\Utils\ProcessException;

class ViewSourceModule extends SmartyModule
{
    public function build($runData)
    {

        $site = $runData->getTemp("site");
        $pageId = $runData->getParameterList()->getParameterValue("pageId");

        $raw = $runData->getParameterList()->getParameterValue("raw");

        if (!$pageId || !is_numeric($pageId)) {
            throw new ProcessException(_("The page cannot be found or does not exist."), "no_page");
        }

        $page = PagePeer::instance()->selectByPrimaryKey($pageId);

        if (!$page || $page->getSiteId() !== $site->getSiteId()) {
            throw new ProcessException(_("The page cannot be found or does not exist."), "no_page");
        }

        $source = $page->getCurrentRevision()->getSourceText();

        $runData->contextAdd("source", $source);
        $runData->contextAdd("raw", $raw);
    }
}
