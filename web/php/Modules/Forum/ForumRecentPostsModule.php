<?php

namespace Wikidot\Modules\Forum;


use Ozone\Framework\Database\Criteria;
use Wikidot\DB\ForumGroupPeer;
use Wikidot\DB\ForumCategoryPeer;

use Wikidot\Utils\CacheableModule;

class ForumRecentPostsModule extends CacheableModule
{

    protected $timeOut = 300;

    public function build($runData)
    {

        $site = $runData->getTemp("site");

        // get forum groups

        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        $c->add("visible", true);
        $c->addOrderAscending("sort_index");

        $groups = ForumGroupPeer::instance()->select($c);

        $res = array();

        foreach ($groups as $g) {
            $c = new Criteria();
            $c->add("group_id", $g->getGroupId());

            $c->addOrderAscending("sort_index");

            $categories = ForumCategoryPeer::instance()->select($c);
            foreach ($categories as $cat) {
                $res[] = array('group' => $g, 'category' => $cat);
            }
        }

        $runData->contextAdd("cats", $res);
    }
}
