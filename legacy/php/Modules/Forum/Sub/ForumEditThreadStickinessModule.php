<?php

namespace Wikidot\Modules\Forum\Sub;


use Ozone\Framework\Database\Database;
use Wikidot\DB\ForumThreadPeer;

use Ozone\Framework\SmartyModule;
use Wikidot\Utils\ProcessException;
use Wikidot\Utils\WDPermissionManager;

class ForumEditThreadStickinessModule extends SmartyModule
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

        $db->commit();
    }
}
