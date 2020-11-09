<?php
use DB\ForumThreadPeer;

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
