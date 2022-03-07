<?php

namespace Wikidot\Modules\Parent;

use Ozone\Framework\SmartyModule;
use Wikidot\Utils\ProcessException;
use Wikidot\Utils\WDPermissionManager;
use Wikijump\Services\Deepwell\Models\Page;

class ParentPageModule extends SmartyModule
{

    public function build($runData)
    {
        $pageId = $runData->getParameterList()->getParameterValue("page_id");

        $page = Page::findIdOnly($pageId);
        if ($page === null || $page->getSiteId() !== $runData->getTemp("site")->getSiteId()) {
            throw new ProcessException(_("Error getting page information."), "no_page");
        }

        $user = $runData->getUser();
        // check permissions now
        $category = $page->getCategory();
        // now check for permissions!!!
        WDPermissionManager::instance()->hasPagePermission('edit', $user, $category, $page);

        if ($page->getParentPageId() !== null) {
            $parentPage = Page::findIdOnly($page->getParentPageId());
            $runData->contextAdd("parentPageName", $parentPage->slug);
        }
    }
}
