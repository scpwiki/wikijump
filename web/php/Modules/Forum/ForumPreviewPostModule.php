<?php

namespace Wikidot\Modules\Forum;

use Ozone\Framework\ODate;
use Wikidot\DB\ForumPost;
use Ozone\Framework\SmartyModule;
use Wikidot\Utils\ProcessException;
use Wikijump\Models\User;

use Wikijump\Services\Wikitext\ParseRenderMode;

use function Wikijump\Services\Wikitext\getWikitextBackend;

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

        $wt = getWikitextBackend(ParseRenderMode::FORUM_POST, null);
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
