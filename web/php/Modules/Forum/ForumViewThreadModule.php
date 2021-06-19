<?php

namespace Wikidot\Modules\Forum;


use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Ozone;
use Wikidot\DB\ForumThreadPeer;
use Wikidot\DB\ForumPostPeer;

use Ozone\Framework\SmartyModule;
use Wikidot\Utils\ProcessException;

class ForumViewThreadModule extends SmartyModule
{

    protected $processPage = true;

    public function render($runData)
    {

        $site = $runData->getTemp("site");
        $pl = $runData->getParameterList();
        $threadId = $pl->getParameterValue("t");

        $parmHash = md5(serialize($pl->asArray()));

        $key = 'forumthread_v..'.$site->getUnixName().'..'.$threadId.'..'.$parmHash;
        $tkey = 'forumthread_lc..'.$site->getUnixName().'..'.$threadId; // last change timestamp
        $akey = 'forumall_lc..'.$site->getUnixName();

        $mc = OZONE::$memcache;
        $struct = $mc->get($key);
        $cacheTimestamp = $struct['timestamp'];
        $changeTimestamp = $mc->get($tkey);
        $allForumTimestamp = $mc->get($akey);
        if ($struct) {
            // check the times

            if ($changeTimestamp && $changeTimestamp <= $cacheTimestamp && $allForumTimestamp && $allForumTimestamp <= $cacheTimestamp) {
                $this->categoryName = $struct['categoryName'];
                $this->threadTitle = $struct['threadTitle'];
                $this->threadId = $struct['threadId'];
                $this->tpage = $struct['tpage'];

                $out = $struct['content'];
                $page = $GLOBALS['page'];

                return $out;
            }
        }

        $out = parent::render($runData);

        // and store the data now
        $struct = array();
        $now = time();
        $struct['timestamp'] = $now;
        $struct['content'] = $out;
        $struct['categoryName']=$this->categoryName;
        $struct['threadTitle']=$this->threadTitle;
        $struct['threadId']=$this->threadId;
        $struct['tpage'] = $this->tpage;

        $mc->set($key, $struct, 0, 864000);

        if (!$changeTimestamp) {
            $changeTimestamp = $now;
            $mc->set($tkey, $changeTimestamp, 0, 864000);
        }
        if (!$allForumTimestamp) {
            $allForumTimestamp = $now;
            $mc->set($akey, $allForumTimestamp, 0, 864000);
        }

        return $out;
    }

    public function build($runData)
    {
        $site = $runData->getTemp("site");
        $pl = $runData->getParameterList();
        $threadId = $pl->getParameterValue("t");

        if ($threadId == null || !is_numeric($threadId)) {
            throw new ProcessException(_("Invalid thread."), "invalid_thread");
        }

        $c = new Criteria();
        $c->add("thread_id", $threadId);
        $c->add("site_id", $site->getSiteId());

        $thread = ForumThreadPeer::instance()->selectOne($c);

        if ($thread == null) {
            throw new ProcessException(_("No thread."), "no_thread");
        }
        $this->threadTitle = $thread->getTitle();
        $this->threadId = $thread->getThreadId();

        $category = $thread->getForumCategory();
        $this->categoryName = $category->getName();

        // check if connected to a page
        $this->tpage = $thread->getPage();

        // get posts

        $c = new Criteria();
        $c->add("thread_id", $threadId);
        $c->add("site_id", $site->getSiteId());
        $c->addJoin("user_id", "users.id");
        $c->addOrderAscending("post_id");

        $posts = ForumPostPeer::instance()->select($c);

        // make a mapping first.
        $map = array();
        $levels = array();

        foreach ($posts as $post) {
            $parentId = $post->getParentId();
            $postId = $post->getPostId();
            if ($parentId === null) {
                // if no parrent - simply add at the end of $map
                $map[] =  $postId;
                $levels[$postId] = 0;
            } else {
                // find a parent

                $cpos = array_search($parentId, $map);
                $clevel = $levels[$parentId];
                // find a place for the post, i.e. the place where level_next == level or the end of array
                $cpos++;
                while (isset($map[$cpos]) && $levels[$map[$cpos]]>$clevel) {
                    $cpos++;
                }
                // insert at this position!!!
                array_splice($map, $cpos, 0, $postId);
                $levels[$postId] = $clevel+1;
            }
        }

        // create container control list

        $cc = array();
        foreach ($map as $pos => $m) {
            // open if previous post has LOWER level
            $clevel = $levels[$m];

            if (isset($map[$pos+1])) {
                $nlevel = $levels[$map[$pos+1]];
                if ($nlevel>$clevel) {
                    $cc[$pos] = 'k';
                }
                if ($nlevel < $clevel) {
                    $cc[$pos]=str_repeat('c', $clevel-$nlevel);
                }
            } else {
                $cc[$pos]=str_repeat('c', $clevel);
            }
        }

        $runData->contextAdd("postmap", $map);
        $runData->contextAdd("levels", $levels);
        $runData->contextAdd("containerControl", $cc);

        $runData->contextAdd("thread", $thread);
        $runData->contextAdd("category", $category);
        $runData->contextAdd("posts", $posts);
        
        $runData->contextAdd("canDelete", WDPermissionManager::instance()->hasForumPermission('moderate_forum', $runData->getUser(), $category));

        $page = $GLOBALS['page'];
    }

    public function processPage($out, $runData)
    {
        // modify title of the page
        if ($this->threadTitle) {
            $pageTitle = $this->categoryName.': '.$this->threadTitle;
            $runData->getTemp("page")->setTitle($pageTitle);
            $out = preg_replace("/<title>(?:.+?)<\/title>/is", "<title>".preg_quote_replacement(htmlspecialchars($pageTitle))."</title>", $out);
        }
        if (!$this->tpage) {
            $ptitle = htmlspecialchars($this->threadTitle);
        } else {
            $ptitle = '<a href="/'.$this->tpage->getUnixName().'">'.htmlspecialchars($this->tpage->getTitle()).'</a> / '._('discussion').'</h1>';
        }

        $out = preg_replace("/<div id=\"page-title\">(?:.*?)<\/div>/is", "<div id=\"page-title\">".preg_quote_replacement($ptitle)."</div>", $out);
        // add rss feed info
        $link = '/feed/forum/t-'.$this->threadId.'.xml';
        $out = preg_replace(
            "/<\/head>/",
            '<link rel="alternate" type="application/rss+xml" title="'._('Posts in the discussion thread').' &quot;'.htmlspecialchars($this->threadTitle).'&quot;" href="'.$link.'"/></head>',
            $out,
            1
        );
        return $out;
    }
}
