<?php
declare(strict_types=1);

namespace Wikidot\Modules\PageTags;

use Ds\Set;
use Ozone\Framework\SmartyModule;
use Wikidot\Utils\ProcessException;
use Wikidot\Utils\WDPermissionManager;
use Wikijump\Services\Deepwell\Models\Page;

class PageTagsModule extends SmartyModule
{
    public function build($runData)
    {
        $user = $runData->getUser();
        $pl = $runData->getParameterList();
        $page_id = $pl->getParameterValue("pageId");
        $site = $runData->getTemp("site");

        if (!is_numeric($page_id)) {
            throw new ProcessException(_("The page cannot be found or does not exist."), "no_page");
        }

        $page = Page::findIdOnly($page_id);
        if ($page === null || $page->getSiteId() != $site->getSiteId()) {
            throw new ProcessException(_("Error getting page information."), "no_page");
        }

        $category = $page->getCategory();

        WDPermissionManager::instance()->hasPagePermission('edit', $user, $category, $page);

        // Fetch the tags and convert them to a string.
        $tags = new Set(); // PagePeer::getTags($page_id);
        $tags = $tags->join(" ");

        $runData->contextAdd("tags", $tags);
    }
}
