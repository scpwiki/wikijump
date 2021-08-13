<?php

namespace Wikidot\Screens\Feed;

use Illuminate\Support\Facades\Cache;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Ozone;
use Wikidot\DB\ForumCategoryPeer;
use Wikidot\DB\ForumThreadPeer;
use Wikidot\Utils\FeedScreen;
use Wikidot\Utils\GlobalProperties;
use Wikidot\Utils\ProcessException;
use Wikijump\Helpers\LegacyTools;
use Wikijump\Models\User;

class ForumCategoryThreadsFeed extends FeedScreen
{

    public function render($runData)
    {
        $site = $runData->getTemp("site");
        $pl = $runData->getParameterList();
        $categoryId = $pl->getParameterValue("c");

        $parmHash = md5(serialize($pl->asArray()));

        $key = 'forumcategorythreads_f..'.$site->getUnixName().'..'.$categoryId.'..'.$parmHash;
        $tkey = 'forumcategory_lc..'.$site->getUnixName().'..'.$categoryId; // last change timestamp
        $akey = 'forumall_lc..'.$site->getUnixName();

        $struct = Cache::get($key);
        $cacheTimestamp = $struct['timestamp'];
        $changeTimestamp = Cache::get($tkey);
        $allForumTimestamp = Cache::get($akey);
        if ($struct) {
            // check the times

            if ($changeTimestamp && $changeTimestamp <= $cacheTimestamp && $allForumTimestamp && $allForumTimestamp <= $cacheTimestamp) {
                return $struct['content'];
            }
        }

        $out = parent::render($runData);

        // and store the data now
        $struct = array();
        $now = time();
        $struct['timestamp'] = $now;
        $struct['content'] = $out;

        Cache::put($key, $struct, 1000);
        if (!$changeTimestamp) {
            $changeTimestamp = $now;
            Cache::put($tkey, $changeTimestamp, 1000);
        }
        if (!$allForumTimestamp) {
            $allForumTimestamp = $now;
            Cache::put($akey, $allForumTimestamp, 10000);
        }

        return $out;
    }

    public function build($runData)
    {

        $site = $runData->getTemp("site");

        $pl = $runData->getParameterList();
        $categoryId = $pl->getParameterValue("c");

        $category = ForumCategoryPeer::instance()->selectByPrimaryKey($categoryId);
        if ($category == null) {
            throw new ProcessException(_("No such category."), "no_category");
        }
        $channel = array();

        $channel['title'] = $category->getName()." (new threads)";
        $channel['link'] = GlobalProperties::$HTTP_SCHEMA . "://" . $site->getDomain()."/forum/c-".$categoryId."/".$category->getUnixifiedName();
        $channel['description'] = _("Threads in the forum category")." \"".$category->getName()."\"";
        if ($category->getDescription()) {
            $channel['description'] .=  " - ".$category->getDescription();
        }

        $items = array();

        $c = new Criteria();
        $c->add("category_id", $categoryId);
        $c->addJoin("user_id", "users.id");
        $c->addOrderDescending("thread_id");
        $c->setLimit(20);
        $threads = ForumThreadPeer::instance()->select($c);

        foreach ($threads as $thread) {
            $item = array();

            $item['title'] = $thread->getTitle();
            $item['link'] = GlobalProperties::$HTTP_SCHEMA . "://" . $site->getDomain()."/forum/t-".$thread->getThreadId().'/'.$thread->getUnixifiedTitle();
            $item['guid'] = GlobalProperties::$HTTP_SCHEMA . "://" . $site->getDomain()."/forum/t-".$thread->getThreadId();
            $item['date'] = date('r', $thread->getDateStarted()->getTimestamp());

            //replace relative links with absolute links!
            $post = $thread->getFirstPost();
            $content =  $post->getText();

            $content = preg_replace('/
                (<.*?)       # Opening tag and its name
                (src|href)   # Attribute selection
                =
                "\/([^"]+)"  # Value of attribute
                ([^>]*>)     # Any other attributes - these must come AFTER src/href
                /six', '\\1\\2="'.GlobalProperties::$HTTP_SCHEMA . "://" . $site->getDomain().'/\\3"\\4', $content);
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

            if ($thread->getDescription()) {
                $item['description'] = $thread->getDescription();
            }

            $item['content'] = $content;
            if (LegacyTools::isSystemAccount($post->getUserId()) === false) {
                $item['authorUserId'] = $post->getUserId();
                $user = $post->getUser();
                $item['author']=$user->username;
            } else {
                $item['author']=$post->getUserString();
            }
            $items[] = $item;
        }

        $runData->contextAdd("channel", $channel);
        $runData->contextAdd("items", $items);
    }
}
