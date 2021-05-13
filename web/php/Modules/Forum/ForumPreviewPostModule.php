<?php

namespace Wikidot\Modules\Forum;

use Ozone\Framework\ODate;
use Ozone\Framework\SmartyModule;
use Wikidot\DB\ForumPost;
use Wikidot\Utils\ProcessException;
use Wikijump\Models\User;
use Wikijump\Services\Wikitext\ParseRenderMode;
use Wikijump\Services\Wikitext\WikitextBackend;

class ForumPreviewPostModule extends SmartyModule
{

    public function build($runData)
    {
        $pl = $runData->getParameterList();
        $title = $pl->getParameterValue("title");
        $description = trim($pl->getParameterValue("description"));
        $source = trim($pl->getParameterValue("source"));

        if ($source == null || $source == '') {
            throw new ProcessException(_("Post is empty."), "post_empty");
        }

        $wt = WikitextBackend::make(ParseRenderMode::FORUM_POST, null);
        $body = $wt->renderHtml($source)->html;

        $post = new ForumPost();
        $post->setText($body);
        $post->setTitle($title);
        $post->setDatePosted(new ODate());

        // now set user_id, user_string

        $userId = $runData->getUserId();
        if ($userId == null) {
            $userString = $runData->createIpString();
        }

        if ($userId) {
            $post->setUserId($userId);
        } else {
            $post->setUserId(User::ANONYMOUS_USER);
            $post->setUserString($userString);
        }

        $runData->contextAdd("post", $post);
    }
}
