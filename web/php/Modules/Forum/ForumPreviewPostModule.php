<?php

namespace Wikidot\Modules\Forum;

use Ozone\Framework\ODate;
use Ozone\Framework\SmartyModule;
use Wikidot\DB\ForumPost;
use Wikidot\Utils\ProcessException;
use Wikijump\Models\User;
use Wikijump\Services\Deepwell\DeepwellService;
use Wikijump\Services\Wikitext\ParseRenderMode;

class ForumPreviewPostModule extends SmartyModule
{

    public function build($runData)
    {
        $pl = $runData->getParameterList();
        $title = $pl->getParameterValue("title");
        $source = trim($pl->getParameterValue("source"));

        if ($source === '') {
            throw new ProcessException(_("Post is empty."), "post_empty");
        }

        $body = DeepwellService::getInstance()->renderHtml(ParseRenderMode::FORUM_POST, $source, null);

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
