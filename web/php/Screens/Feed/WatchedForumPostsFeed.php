<?php

namespace Wikidot\Screens\Feed;

use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Ozone;
use Wikidot\DB\ForumPostPeer;
use Wikidot\Utils\FeedScreen;
use Wikidot\Utils\GlobalProperties;
use Wikidot\Utils\WDRenderUtils;

class WatchedForumPostsFeed extends FeedScreen
{

    protected $requiresAuthentication = true;

    public function render($runData)
    {
        $user = $runData->getTemp("user");
        $key = "watchedforum..".$user->getUserId();
        $mc = OZONE::$memcache;
        $out = $mc->get($key);
        if ($out) {
            return $out;
        }
        $out = parent::render($runData);
        $mc->set($key, $out, 0, 600);
        return $out;
    }

    public function build($runData)
    {

        $user = $runData->getTemp("user");
        $userId = $user->getUserId();

        // set language for the user
        $lang = $user->getLanguage();
        $runData->setLanguage($lang);
        $GLOBALS['lang'] = $lang;

        // and for gettext too:

        switch ($lang) {
            case 'pl':
                $glang="pl_PL";
                break;
            case 'en':
                $glang="en_US";
                break;
        }

        putenv("LANG=$glang");
        putenv("LANGUAGE=$glang");
        setlocale(LC_ALL, $glang.'.UTF-8');

        // now just get watched page changes for the user...

        $c = new Criteria();

        $c->addJoin("thread_id", "forum_thread.thread_id");
        $c->addJoin("thread_id", "watched_forum_thread.thread_id");
        $c->addJoin("user_id", "ozone_user.user_id");
        $c->add("watched_forum_thread.user_id", $user->getUserId());
        $c->addOrderDescending("post_id");
        $c->setLimit(30);

        $posts = ForumPostPeer::instance()->select($c);

        $channel['title'] = _('Wikijump.com watched forum discussions for user').' "'.$user->getNickName().'"';
        $channel['link'] = GlobalProperties::$HTTP_SCHEMA . "://" . GlobalProperties::$URL_HOST . "/account:you/start/watched-forum";

        $items = array();

        foreach ($posts as $post) {
            $thread = $post->getForumThread();

            $site = $post->getSite();

            $item = array();

            $item['title'] = $post->getTitle() . ' ('._('on site').' "'.htmlspecialchars($site->getName()).'")';
            $item['link'] = GlobalProperties::$HTTP_SCHEMA . "://" . $site->getDomain()."/forum/t-".$thread->getThreadId().'/'.$thread->getUnixifiedTitle().'#post-'.$post->getPostId();
            $item['guid'] = GlobalProperties::$HTTP_SCHEMA . "://" . $site->getDomain()."/forum/t-".$thread->getThreadId().'#post-'.$post->getPostId();

            $item['date'] = date('r', $post->getDatePosted()->getTimestamp());

            $content =  $post->getText();

            $content = preg_replace(
                '/
                (<.*?)       # Opening tag and its name
                (src|href)   # Attribute selection
                =
                "\/([^"]+)"  # Value of attribute
                ([^>]*>)     # Any other attributes - these must come AFTER src/href
                /six',
                '\\1\\2="'.GlobalProperties::$HTTP_SCHEMA . "://" . $site->getDomain().'/\\3"\\4',
                $content
            );
            # Remove any script elements
            $content = preg_replace(
                '/
                <script\s+[^>]+>
                .*?
                <\/script>
                /isx',
                '',
                $content
            );
            # Strip out any attribute that starts with "on" from any element
            $content = preg_replace(
                '/
                (<[^>]*\s+)
                on[a-z]+="[^"]+"
                ([^>]*>)
                /six',
                '\\1 \\2',
                $content
            );

            // add Extra info.

            $content .= '<br/><hr/>';
            $content .= _('Site').': <a href="'.GlobalProperties::$HTTP_SCHEMA . "://" . $site->getDomain().'">'.htmlspecialchars($site->getName()).'</a><br/>';
            $content .= _('Forum category').': <a href="'.GlobalProperties::$HTTP_SCHEMA . "://" . $site->getDomain().'/forum/c-'.$thread->getCategoryId().'">'.htmlspecialchars($thread->getForumCategory()->getName()).'</a><br/>';
            $content .= _('Forum thread').': <a href="'.GlobalProperties::$HTTP_SCHEMA . "://" . $site->getDomain().'/forum/t-'.$thread->getThreadId().'/'.$thread->getUnixifiedTitle().'">'
                .htmlspecialchars($thread->getTitle()).'</a><br/>';
            $content .= _('Author of the post').': '.WDRenderUtils::renderUser($post->getUserOrString()).'<br/>';

            $item['content'] = $content;
            if ($post->getUserId()>0) {
                $item['authorUserId'] = $post->getUserId();
                $user = $post->getUser();
                $item['author']=$user->getNickName();
            } else {
                $item['author']=$post->getUserString();
            }

            $items[] = $item;
        }

        $runData->contextAdd("channel", $channel);
        $runData->contextAdd("items", $items);
    }
}
