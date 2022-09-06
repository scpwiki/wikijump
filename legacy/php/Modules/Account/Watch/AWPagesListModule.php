<?php

namespace Wikidot\Modules\Account\Watch;

use Ozone\Framework\Database\Criteria;
use Wikidot\Utils\AccountBaseModule;

class AWPagesListModule extends AccountBaseModule
{

    public function build($runData)
    {

        $user = $runData->getUser();
        $runData->contextAdd("user", $user);

        $pl = $runData->getParameterList();

        // get watched pages for this user

        $c = new Criteria();

        $q = "SELECT page.* FROM watched_page, page " .
                "WHERE watched_page.user_id='".$user->id."' " .
                        "AND watched_page.page_id=page.page_id";
        $c->setExplicitQuery($q);

        $pages = [null]; // TODO run query

        $runData->contextAdd("pages", $pages);

        $runData->contextAdd("pagesCount", count($pages));
    }
}
