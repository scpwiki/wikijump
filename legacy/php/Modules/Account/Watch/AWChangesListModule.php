<?php

namespace Wikidot\Modules\Account\Watch;




use Ozone\Framework\Database\Criteria;
use Wikidot\DB\PageRevisionPeer;
use Wikidot\Utils\AccountBaseModule;

class AWChangesListModule extends AccountBaseModule
{

    public function build($runData)
    {

        $user = $runData->getUser();

        $pl = $runData->getParameterList();

        $pageNumber = $pl->getParameterValue("page");
        if ($pageNumber === null) {
            $pageNumber = 1;
        }
        $limit = $pl->getParameterValue("limit");

        if ($limit == null || !is_numeric($limit) || $limit > 20) {
            $limit = 20;
        }
        $perPage = $limit;
        $offset = ($pageNumber - 1)*$perPage;
        $count = $perPage*2 + 1;

        // join the tables:
        // watched_page, page_revision, user, page, site. ???

        $c = new Criteria();

        $c->addJoin("page_id", "page.page_id");
        $c->addJoin("page_id", "watched_page.page_id");
        $c->addJoin("user_id", "users.id");
        $c->add("watched_page.user_id", $user->id);
        $c->addOrderDescending("page_revision.revision_id");
        $c->setLimit($count, $offset);

        $revisions = PageRevisionPeer::instance()->select($c);

        $counted = count($revisions);
        $pagerData = array();
        $pagerData['currentPage'] = $pageNumber;
        if ($counted >$perPage*2) {
            $knownPages=$pageNumber + 2;
            $pagerData['knownPages'] = $knownPages;
        } elseif ($counted >$perPage) {
            $knownPages=$pageNumber + 1;
            $pagerData['totalPages'] = $knownPages;
        } else {
            $totalPages = $pageNumber;
            $pagerData['totalPages'] = $totalPages;
        }
        $revisions = array_slice($revisions, 0, $perPage);

        $runData->contextAdd("pagerData", $pagerData);

        $runData->contextAdd("revisions", $revisions);
    }
}
