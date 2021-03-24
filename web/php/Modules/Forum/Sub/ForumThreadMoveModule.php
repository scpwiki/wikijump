<?php

namespace Wikidot\Modules\Forum\Sub;


use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Database\Database;
use Wikidot\DB\ForumThreadPeer;
use Wikidot\DB\ForumGroupPeer;
use Wikidot\DB\ForumCategoryPeer;

use Ozone\Framework\SmartyModule;
use Wikidot\Utils\ProcessException;
use Wikidot\Utils\WDPermissionManager;

class ForumThreadMoveModule extends SmartyModule
{

    public function build($runData)
    {
        $pl = $runData->getParameterList();

        $threadId = $pl->getParameterValue("threadId");
        $site = $runData->getTemp("site");

        $db = Database::connection();
        $db->begin();

        $thread = ForumThreadPeer::instance()->selectByPrimaryKey($threadId);
        if ($thread == null || $thread->getSiteId() !== $site->getSiteId()) {
            throw new ProcessException(_("No thread found... Is it deleted?"), "no_thread");
        }

        $category = $thread->getForumCategory();
        WDPermissionManager::instance()->hasForumPermission('moderate_forum', $runData->getUser(), $category);

        $runData->contextAdd("thread", $thread);
        $runData->contextAdd("category", $thread->getForumCategory());

        // and select categories to move into too.

        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        $c->addOrderDescending("visible");
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

        $runData->contextAdd("categories", $res);

        $db->commit();
    }
}
