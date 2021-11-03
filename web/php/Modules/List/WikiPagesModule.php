<?php

namespace Wikidot\Modules\List;

use Ozone\Framework\Database\Criteria;
use Wikidot\DB\CategoryPeer;
use Wikidot\DB\PagePeer;

use Wikidot\Utils\CacheableModule;
use Wikidot\Utils\ProcessException;

class WikiPagesModule extends CacheableModule
{

    protected $timeOut = 10;

    public function build($runData)
    {

        $pl = $runData->getParameterList();
        $site = $runData->getTemp("site");

        $categoryName = $pl->getParameterValue("category", "MODULE", "AMODULE");
        $details = $pl->getParameterValue("details", "MODULE", "AMODULE");

        $order = $pl->getParameterValue("order", "MODULE", "AMODULE");
        $limit = $pl->getParameterValue("limit", "MODULE", "AMODULE");

        if ($categoryName !== null) {
            $category = CategoryPeer::instance()->selectByName($categoryName, $site->getSiteId());
            if ($category == null) {
                throw new ProcessException(_("The category cannot be found."));
            }
        }

        // now select pages according to the specified criteria

        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        if ($category) {
            $c->add("category_id", $category->getCategoryId());
        }

        switch ($order) {
            case 'dateCreatedDesc':
                $c->addOrderDescending('page_id');
                break;
            case 'dateCreatedAsc':
                $c->addOrderAscending('page_id');
                break;
            case 'dateEditedDesc':
                $c->addOrderDescending('date_last_edited');
                break;
            case 'dateEditedAsc':
                $c->addOrderAscending('date_last_edited');
                break;
            case 'titleDesc':
                $c->addOrderDescending("COALESCE(title, unix_name)");
                break;
            default:
                $c->addOrderAscending("COALESCE(title, unix_name)");
        }

        if ($limit && is_numeric($limit) && $limit > 0) {
            $c->setLimit($limit);
        }

        $pages = PagePeer::instance()->select($c);

        // by default cathegorize by first letter...

        $runData->contextAdd("pages", $pages);
        $runData->contextAdd("details", $details);
    }
}
