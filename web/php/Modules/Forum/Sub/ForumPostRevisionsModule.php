<?php

namespace Wikidot\Modules\Forum\Sub;


use Ozone\Framework\Database\Criteria;
use Wikidot\DB\ForumPostPeer;
use Wikidot\DB\ForumPostRevisionPeer;

use Ozone\Framework\SmartyModule;
use Wikidot\Utils\ProcessException;

class ForumPostRevisionsModule extends SmartyModule
{

    public function build($runData)
    {
        $pl = $runData->getParameterList();
        $postId = $pl->getParameterValue("postId");

        $site = $runData->getTemp("site");

        if ($postId == null || !is_numeric($postId)) {
            throw new ProcessException(_("No post specified."), "no_post");
        }

        $post = ForumPostPeer::instance()->selectByPrimaryKey($postId);
        if ($post == null || $post->getSiteId() != $site->getSiteId()) {
            throw new ProcessException(_("No post specified."), "no_post");
        }

        // get all revisions

        $c = new Criteria();
        $c->add("post_id", $postId);
        $c->addOrderDescending("revision_id");

        $revs = ForumPostRevisionPeer::instance()->select($c);

        $runData->contextAdd("revisions", $revs);
        $runData->contextAdd("post", $post);

        $runData->ajaxResponseAdd("postId", $postId);
    }
}
