<?php

namespace Wikidot\Modules\Forum;


use Illuminate\Support\Facades\Cache;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\ODate;
use Ozone\Framework\Ozone;
use Wikidot\DB\PagePeer;
use Wikidot\DB\ForumThreadPeer;
use Wikidot\DB\ForumCategoryPeer;
use Wikidot\DB\ForumCategory;
use Wikidot\DB\ForumGroupPeer;
use Wikidot\DB\ForumGroup;
use Wikidot\DB\ForumThread;
use Wikidot\DB\ForumPostPeer;

use Ozone\Framework\SmartyModule;
use Wikidot\Utils\GlobalProperties;
use Wikidot\Utils\ProcessException;
use Wikijump\Models\User;

class ForumCommentsListModule extends SmartyModule
{

    private $threadId;

    protected $processPage = false;

    public function render($runData)
    {
        $site = $runData->getTemp("site");

        $pl = $runData->getParameterList();
        $pageId = $pl->getParameterValue("pageId");
        $pageName = $runData->getTemp("pageUnixName");

        $parmHash = md5(serialize($pl->asArray()));

        if ($pageId !== null) {
            $key = 'pagecomments_v_pageid..'.$site->getSlug().'..'.$pageId.'..'.$parmHash;
        } else {
            $key = 'pagecomments_v_pagename..'.$site->getSlug().'..'.$pageName.'..'.$parmHash;
        }
        $akey = 'forumall_lc..'.$site->getSlug();

        $uri = GlobalProperties::$MODULES_JS_URL.'/forum/ForumViewThreadModule.js';
        $this->extraJs[] = $uri;

        $struct = Cache::get($key);
        $allForumTimestamp = Cache::get($akey);
        if ($struct) {
            // check the times
            $cacheTimestamp = $struct['timestamp'];
            $threadId = $struct['threadId'];
            $tkey = 'forumthread_lc..'.$site->getSlug().'..'.$threadId; // last change timestamp
            $changeTimestamp = Cache::get($tkey);
            if ($changeTimestamp && $changeTimestamp <= $cacheTimestamp && $allForumTimestamp && $allForumTimestamp <= $cacheTimestamp) {
                $runData->ajaxResponseAdd("threadId", $threadId);
                return $struct['content'];
            }
        }

        $out = parent::render($runData);

        // and store the data now
        $struct = array();
        $now = time();
        $struct['timestamp'] = $now;
        $struct['content'] = $out;
        $struct['threadId']=$this->threadId;

        if (!$changeTimestamp) {
            $tkey = 'forumthread_lc..'.$site->getSlug().'..'.$this->threadId; // last change timestamp
            $changeTimestamp = Cache::get($tkey);
        }

        Cache::put($key, $struct, 864000);
        if (!$changeTimestamp) {
            $tkey = 'forumthread_lc..'.$site->getSlug().'..'.$this->threadId;
            $changeTimestamp = $now;
            Cache::put($tkey, $changeTimestamp, 864000);
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
        $page = $runData->getTemp("page");

        $pl = $runData->getParameterList();

        if ($page == null) {
            $pageId = $pl->getParameterValue("pageId");
            if ($pageId !== null && is_numeric($pageId)) {
                $page = PagePeer::instance()->selectByPrimaryKey($pageId);
            } else {
                $pageName = $runData->getTemp("pageUnixName");

                $site = $runData->getTemp("site");
                $page =  PagePeer::instance()->selectByName($site->getSiteId(), $pageName);
            }

            if ($page == null || $page->getSiteId() !== $site->getSiteId()) {
                throw new ProcessException(_("Can not find related page."), "no_page");
            }
        }

        // check for a discussion thread. if not exists - create it!

        $c = new Criteria();
        $c->add("page_id", $page->getPageId());
        $c->add("site_id", $site->getSiteId());

        $thread = ForumThreadPeer::instance()->selectOne($c);

        if ($thread == null) {
            // create thread!!!
            $c = new Criteria();
            $c->add("site_id", $site->getSiteId());
            $c->add("per_page_discussion", true);

            $category = ForumCategoryPeer::instance()->selectOne($c);

            if ($category == null) {
                // create this category!
                $category = new ForumCategory();
                $category->setName(_("Per page discussions"));
                $category->setDescription(_("This category groups discussions related to particular pages within this site."));
                $category->setPerPageDiscussion(true);
                $category->setSiteId($site->getSiteId());

                // choose group. create one?
                $c = new Criteria();
                $c->add("site_id", $site->getSiteId());
                $c->add("name", "Hidden");
                $group = ForumGroupPeer::instance()->selectOne($c);
                if ($group == null) {
                    $group = new ForumGroup();
                    $group->setName(_("Hidden"));
                    $group->setDescription(_("Hidden group used for storing some discussion threads."));
                    $group->setSiteId($site->getSiteId());
                    $group->setVisible(false);
                    $group->save();
                }
                $category->setGroupId($group->getGroupId());
                $category->save();
            }

            // now create thread...
            $thread = new ForumThread();
            $thread->setCategoryId($category->getCategoryId());
            $thread->setSiteId($site->getSiteId());
            $thread->setPageId($page->getPageId());
            $thread->setUserId(User::AUTOMATIC_USER);
            $thread->setDateStarted(new ODate());
            $thread->setNumberPosts(0);
            $thread->save();

            $page->setThreadId($thread->getThreadId());
            $page->save();

            $category->setNumberThreads($category->getNumberThreads()+1);
            $category->save();
        } else {
            $category = $thread->getForumCategory();
        }

        $this->threadId = $thread->getThreadId();

        $c = new Criteria();
        $c->add("thread_id", $thread->getThreadId());
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

        $runData->ajaxResponseAdd("threadId", $thread->getThreadId());
    }
}
