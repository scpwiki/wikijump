<?php
use DB\PagePeer;

class PageFilesModule extends SmartyModule
{

    public function build($runData)
    {
        $site = $runData->getTemp("site");

        $pageId = $runData->getParameterList()->getParameterValue("page_id");
        if (!$pageId || !is_numeric($pageId)) {
            throw new ProcessException(_("The page cannot be found or does not exist."), "no_page");
        }
        $page = PagePeer::instance()->selectByPrimaryKey($pageId);
        if (!$page || $page->getSiteId() !== $site->getSiteId()) {
            throw new ProcessException(_("The page cannot be found or does not exist."), "no_page");
        }
        $files = $page->getFiles();

        if (count($files)>0) {
            $runData->contextAdd("files", $files);
            $runData->contextAdd("filePath", "/local--files/".$page->getUnixName()."/");
            $totalPageSize = FileHelper::totalPageFilesSize($pageId);
            $totalPageSize = FileHelper::formatSize($totalPageSize);
            $runData->contextAdd("totalPageSize", $totalPageSize);
        }
    }
}
