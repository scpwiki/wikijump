<?php

namespace Wikidot\Modules\XList;

use Ozone\Framework\Database\Criteria;
use Wikidot\DB\PagePeer;

use Ozone\Framework\SmartyModule;
use Wikidot\Utils\ProcessException;

class ChildPagesModule extends SmartyModule
{

    public function build($runData)
    {
        $page = $runData->getTemp("page");
        if (!$page) {
            $pageName = $runData->getTemp("pageUnixName");
            $site = $runData->getTemp("site");
            $page =  PagePeer::instance()->selectByName($site->getSiteId(), $pageName);
        }

        if (!$page) {
            throw new ProcessException(_("Unable to retrieve page data."), "no_page");
        }

        $c = new Criteria();
        $c->add("parent_page_id", $page->getPageId());
        $c->addOrderAscending("COALESCE(title, unix_name)");

        $pages = PagePeer::instance()->select($c);
        if (count($pages)>0) {
            $runData->contextAdd("pages", $pages);
        }
    }
}
