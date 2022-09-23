<?php

namespace Wikidot\Modules\Forum\Sub;


use Ozone\Framework\Database\Criteria;
use Wikidot\DB\ForumPostPeer;
use Wikidot\DB\ModeratorPeer;
use Wikidot\DB\AdminPeer;

use Ozone\Framework\SmartyModule;
use Wikidot\Utils\ProcessException;
use Wikidot\Utils\WDPermissionException;
use Wikidot\Utils\WDPermissionManager;

class ForumEditPostFormModule extends SmartyModule
{

    public function build($runData)
    {
        $pl = $runData->getParameterList();
        $postId = $pl->getParameterValue("postId", "AMODULE");
        $user = $runData->getUser();
        $site = $runData->getTemp("site");

        if ($postId == null || !is_numeric($postId)) {
            throw new ProcessException(_("No post specified."), "no_post");
        }

        $post = ForumPostPeer::instance()->selectByPrimaryKey($postId);
        if ($post == null || $post->getSiteId() != $site->getSiteId()) {
            throw new ProcessException(_("No post specified."), "no_post");
        }

        $category = $post->getForumThread()->getCategory();
        WDPermissionManager::instance()->hasForumPermission('edit_post', $runData->getUser(), $category, null, $post);

        // check if thread blocked
        $thread = $post->getForumThread();
        if ($thread->getBlocked()) {
            // check if moderator or admin
            $c = new Criteria();
            $c->add("site_id", $site->getSiteId());
            $c->add("user_id", $user->id);
            $rel = ModeratorPeer::instance()->selectOne($c);
            if (!$rel || strpos($rel->getPermissions(), 'f') == false) {
                $rel = AdminPeer::instance()->selectOne($c);
                if (!$rel) {
                    throw new WDPermissionException(_("Sorry, this thread is blocked. Nobody can add new posts nor edit existing ones."));
                }
            }
        }

        // OK for now...

        // keep the session - i.e. put an object into session storage not to delete it!!!
        $runData->sessionAdd("keep", true);

        $runData->contextAdd("post", $post);

        $runData->ajaxResponseAdd("postId", $postId);

        $userId = $runData->getUserId();
        if ($userId == null) {
            $userString = $runData->createIpString();
            $runData->contextAdd("anonymousString", $userString);
        }
    }
}
