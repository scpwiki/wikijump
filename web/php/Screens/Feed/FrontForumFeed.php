<?php

namespace Wikidot\Screens\Feed;

use Illuminate\Support\Facades\Cache;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Ozone;
use Wikidot\DB\PagePeer;
use Wikidot\DB\FrontForumFeedPeer;
use Wikidot\DB\ForumCategoryPeer;
use Wikidot\DB\ForumThreadPeer;
use Wikidot\Utils\FeedScreen;
use Wikidot\Utils\GlobalProperties;
use Wikidot\Utils\ProcessException;
use Wikijump\Helpers\LegacyTools;
use Wikijump\Models\User;

class FrontForumFeed extends FeedScreen
{

    public function render($runData)
    {
        $site = $runData->getTemp('site');
        $pl = $runData->getParameterList();

        $pageName = $pl->getParameterValue('page');
        $label = $pl->getParameterValue('label');

        $key = 'frontforumfeed..'.$site->getUnixName().'..'.$pageName.'..'.$label;

        $valid = true;

        $struct = Cache::get($key);
        if (!$struct) {
            $valid = false;
        }
        $cacheTimestamp = $struct['timestamp'];

        $fkey = 'frontforumfeedobject..' .$site->getUnixName().'..'.$pageName.'..'.$label;
        $feed = Cache::get($fkey);

        if (!$feed) {
            $page = PagePeer::instance()->selectByName($site->getSiteId(), $pageName);

            //  get the feed object
            $c = new Criteria();
            $c->add('page_id', $page->getPageId());
            $c->add('label', $label);

            $feed = FrontForumFeedPeer::instance()->selectOne($c);
            Cache::put($fkey, $feed, 3600);
        }

        $now = time();

        $categoryIds = $feed->getCategories();

        // now check lc for ALL categories involved
        $cats = preg_split('/[,;] ?/', $categoryIds);

        foreach ($cats as $cat) {
            $tkey = 'forumcategory_lc..'.$site->getUnixName().'..'.$cat; // last change timestamp
            $changeTimestamp = Cache::get($tkey);
            if ($changeTimestamp && $cacheTimestamp && $changeTimestamp <= $cacheTimestamp) {
                //cache valid
            } else {
                $valid = false;
                if (!$changeTimestamp) {
                    //  put timestamp
                    Cache::put($tkey, $now, 10000);
                    $valid = false;
                }
            }
        }
        $akey = 'forumall_lc..'.$site->getUnixName();
        $allForumTimestamp = Cache::get($akey);
        if ($allForumTimestamp &&  $cacheTimestamp && $changeTimestamp <= $cacheTimestamp) {
            //cache valid
        } else {
            if (!$allForumTimestamp) {
                Cache::put($akey, $now, 10000);
            }
        }

        if ($valid) {
            return $struct['content'];
        }

        $out = parent::render($runData);

        // and store the data now
        $struct = array();
        $now = time();
        $struct['timestamp'] = $now;
        $struct['content'] = $out;

        Cache::put($key, $struct, 1000);

        return $out;
    }

    public function build($runData)
    {

        $site = $runData->getTemp('site');

        $pl = $runData->getParameterList();

        $pageName = $pl->getParameterValue('page');
        $label = $pl->getParameterValue('label');

        // get the feed object
        $page = PagePeer::instance()->selectByName($site->getSiteId(), $pageName);
        if (!$page) {
            throw new ProcessException(_('No such page.'), 'no_page');
        }
        $c = new Criteria();
        $c->add('page_id', $page->getPageId());
        $c->add('label', $label);
        $feed = FrontForumFeedPeer::instance()->selectOne($c);

        $categoryIds = $feed->getCategories();
        $cats = preg_split('/[,;] ?/', $categoryIds);

        $ccat = new Criteria();
        $categories = array();

        // get page
        $page = PagePeer::instance()->selectByPrimaryKey($feed->getPageId());
        if (!$page) {
            throw new ProcessException(_('Page cannot be found.'), 'no_page');
        }

        foreach ($cats as $categoryId) {
            if ($categoryId === null || !is_numeric($categoryId)) {
                throw new ProcessException(_('Problem parsing attribute "category".'), 'no_category');
            }

            $category = ForumCategoryPeer::instance()->selectByPrimaryKey($categoryId);

            if ($category == null || $category->getSiteId() !== $site->getSiteId()) {
                throw new ProcessException(_('Requested forum category does not exist.'), 'no_category');
            }

            $categories[$category->getCategoryId()] = $category;
            $ccat->addOr('category_id', $category->getCategoryId());
        }
        $c = new Criteria();
        $c->addCriteriaAnd($ccat);

        $c->addOrderDescending('thread_id');
        $c->setLimit(30);
        $threads = ForumThreadPeer::instance()->select($c);

        $channel['title'] = $feed->getTitle();
        $channel['link'] = GlobalProperties::$HTTP_SCHEMA . '://' . $site->getDomain(). '/' .$page->getUnixName();
        if ($feed->getDescription()) {
            $channel['description'] = $feed->getDescription();
        }

        $items = array();

        foreach ($threads as $thread) {
            $item = array();

            $item['title'] = $thread->getTitle();
            $item['link'] = GlobalProperties::$HTTP_SCHEMA . '://' . $site->getDomain(). '/forum/t-' .$thread->getThreadId().'/'.$thread->getUnixifiedTitle();
            $item['guid'] = $item['link'];
            $item['date'] = date('r', $thread->getDateStarted()->getTimestamp());

            $item['category'] = $thread->getCategory()->getName();

            //replace relative links with absolute links!
            $post = $thread->getFirstPost();
            if (!$post) {
                continue;
            }

            $content =  $post->getText();

            $content = preg_replace(';(<.*?)(src|href)="/([^"]+)"([^>]*>);si', '\\1\\2="'.GlobalProperties::$HTTP_SCHEMA . '://' . $site->getDomain().'/\\3"\\4', $content);
            $content = preg_replace(';<script\s+[^>]+>.*?</script>;is', '', $content);
            $content = preg_replace(';(<[^>]*\s+)on[a-z]+="[^"]+"([^>]*>);si', '\\1 \\2', $content);

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

        $runData->contextAdd('channel', $channel);
        $runData->contextAdd('items', $items);
    }
}
