<?php

namespace Wikidot\Modules\Edit;


use Wikidot\DB\PagePeer;
use Wikidot\DB\CategoryPeer;

use Ozone\Framework\SmartyModule;
use Wikidot\Utils\ProcessException;
use Wikidot\Utils\WikiTransformation;

class PagePreviewModule extends SmartyModule
{

    public function build($runData)
    {
        $pl = $runData->getParameterList();
        $source = $pl->getParameterValue("source");
        $mode = $pl->getParameterValue("mode");

        $site = $runData->getTemp("site");

        // TODO remove this WikiTransformation
        // page previews are going to happen in the new editor, so this is moot
        $wt = new WikiTransformation();
        $pageId = $pl->getParameterValue("pageId");
        if ($pageId) {
            $runData->setTemp("pageId", $pageId);
            $page = PagePeer::instance()->selectByPrimaryKey($pageId);
            if ($page == null || $page->getSiteId() != $site->getSiteId()) {
                throw new ProcessException(_("Error selecting the page."));
            }
            $runData->setTemp("page", $page);
            $wt->setPage($page);
        } else {
            // TODO remove this and replace with regular PageInfo
            $wt->setPageSlug($pl->getParameterValue('page_unix_name'));
        }

        /* Get the category and apply the "live template" to the source. */
        $pageUnixName = $pl->getParameterValue("page_unix_name");

        if (strpos($pageUnixName, ":") != false) {
            $tmp0 = explode(':', $pageUnixName);
            $categoryName = $tmp0[0];
        } else {
            $categoryName = "_default";
        }

        $category = CategoryPeer::instance()->selectByName($categoryName, $site->getSiteId());

        /* Look for the template (if any). */
        if (!preg_match('/(:|^)_/', $pageUnixName)) {
            $templatePage = PagePeer::instance()->selectByName(
                $site->getSiteId(),
                ($categoryName == '_default' ? '' : $categoryName.':') .'_template'
            );

            if ($templatePage) {
                $source = $wt->assemblyTemplate($source, $templatePage->getSource());
            }
        }

        $result = $wt->processSource($source);

        $body = $result;
        $runData->contextAdd("body", $body);
        $runData->ajaxResponseAdd("title", $pl->getParameterValue("title"));
    }
}
