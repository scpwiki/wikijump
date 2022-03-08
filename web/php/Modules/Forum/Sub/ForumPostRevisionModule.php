<?php

namespace Wikidot\Modules\Forum\Sub;

use Ozone\Framework\SmartyModule;
use Wikidot\DB\ForumPostRevisionPeer;
use Wikidot\Utils\ProcessException;
use Wikijump\Services\Deepwell\DeepwellService;
use Wikijump\Services\Wikitext\ParseRenderMode;

class ForumPostRevisionModule extends SmartyModule
{

    public function build($runData)
    {
        $pl = $runData->getParameterList();
        $revisionId = $pl->getParameterValue("revisionId");

        if ($revisionId == null || !is_numeric($revisionId)) {
            throw new ProcessException(_("No revision specified."), "no_post");
        }

        $revision = ForumPostRevisionPeer::instance()->selectByPrimaryKey($revisionId);
        if ($revision == null) {
            throw new ProcessException(_("No revision specified."), "no_post");
        }

        $runData->ajaxResponseAdd("title", $revision->getTitle());

        $source = $revision->getText();
        $body = DeepwellService::getInstance()->renderHtml(ParseRenderMode::FORUM_POST, $source, null);

        $runData->ajaxResponseAdd("content", $body);
        $runData->ajaxResponseAdd("postId", $revision->getPostId());
    }
}
