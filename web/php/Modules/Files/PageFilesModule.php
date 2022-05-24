<?php

namespace Wikidot\Modules\Files;

use Ozone\Framework\Database\Criteria;
use Ozone\Framework\SmartyModule;
use Wikidot\Utils\FileHelper;
use Wikidot\Utils\ProcessException;
use Wikijump\Services\Deepwell\Models\File;
use Wikijump\Services\Deepwell\Models\Page;

class PageFilesModule extends SmartyModule
{

    public function build($runData)
    {
        $site = $runData->getTemp("site");

        $pageId = $runData->getParameterList()->getParameterValue("page_id");
        if (!$pageId || !is_numeric($pageId)) {
            throw new ProcessException(_("The page cannot be found or does not exist."), "no_page");
        }

        $page = Page::findIdOnly($pageId);
        if ($page === null || $page->getSiteId() !== $site->getSiteId()) {
            throw new ProcessException(_("The page cannot be found or does not exist."), "no_page");
        }

        $files = File::findFromPage($pageId);

        if (count($files) > 0) {
            $runData->contextAdd("files", $files);
            $runData->contextAdd("filePath", "/local--files/".$page->slug."/");
            $totalPageSize = FileHelper::totalPageFilesSize($pageId);
            $totalPageSize = FileHelper::formatSize($totalPageSize);
            $runData->contextAdd("totalPageSize", $totalPageSize);
        }
    }
}
