<?php
use DB\ForumThreadPeer;
use DB\ModeratorPeer;
use DB\AdminPeer;

class ForumEditThreadMetaModule extends SmartyModule
{

    public function build($runData)
    {
        $pl = $runData->getParameterList();

        $threadId = $pl->getParameterValue("threadId");
        $site = $runData->getTemp("site");
        $user = $runData->getUser();

        $db = Database::connection();
        $db->begin();

        $thread = ForumThreadPeer::instance()->selectByPrimaryKey($threadId);

        if ($thread == null || $thread->getSiteId() !== $site->getSiteId()) {
            throw new ProcessException(_("No thread found... Is it deleted?"), "no_thread");
        }

        // check if thread blocked
        if ($thread->getBlocked()) {
            // check if moderator or admin
            $c = new Criteria();
            $c->add("site_id", $site->getSiteId());
            $c->add("user_id", $user->getUserId());
            $rel = ModeratorPeer::instance()->selectOne($c);
            if (!$rel || strpos($rel->getPermissions(), 'f') == false) {
                $rel = AdminPeer::instance()->selectOne($c);
                if (!$rel) {
                    throw new WDPermissionException(_("Sorry, this thread is blocked. Nobody can add new posts nor edit existing ones."));
                }
            }
        }

        $category = $thread->getCategory();
        WDPermissionManager::instance()->hasForumPermission('edit_thread', $runData->getUser(), $category, $thread);

        $runData->contextAdd("thread", $thread);

        $db->commit();
    }
}
