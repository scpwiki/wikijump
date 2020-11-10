<?php
use DB\ForumPostPeer;

class ForumDeletePostModule extends SmartyModule
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
        try {
            WDPermissionManager::instance()->hasForumPermission('moderate_forum', $runData->getUser(), $category);
        } catch (Exception $e) {
            throw new WDPermissionException(_("Sorry, you are not allowed to delete posts. Only site administrators and moderators are the ones who can."));
        }

        // OK for now...
        //check if there any child posts

        $c = new Criteria();
        $c->add("parent_id", $postId);
        $chpc =  ForumPostPeer::instance()->selectCount($c);

        if ($chpc>0) {
            $runData->contextAdd("hasChildren", true);
        }

        $runData->contextAdd("post", $post);

        $runData->ajaxResponseAdd("postId", $postId);
    }
}
