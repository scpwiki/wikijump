<?php
namespace DB;

use \WDStringUtils;
use Criteria;

/**
 * Object Model class.
 *
 */
class ForumThread extends ForumThreadBase
{

    private $page;

    public function getUnixifiedTitle()
    {
        return WDStringUtils::toUnixName($this->getTitle());
    }

    public function getTitle()
    {
        $pageId = $this->getPageId();
        if ($pageId == null) {
            return parent::getTitle();
        } else {
            $page = $this->getPage();
            return $page->getTitle();
        }
    }

    public function getPage()
    {
        if ($this->page) {
            return $this->page;
        } else {
            if ($this->getPageId() === null) {
                return null;
            }
            $page = PagePeer::instance()->selectByPrimaryKey($this->getPageId());
            $this->page = $page;
            return $page;
        }
    }

    public function getUser()
    {
        if ($this->getUserId() == 0) {
            return null;
        }
        if (is_array($this->prefetched)) {
            if (in_array('ozone_user', $this->prefetched)) {
                if (in_array('ozone_user', $this->prefetchedObjects)) {
                    return $this->prefetchedObjects['ozone_user'];
                } else {
                    $obj = new OzoneUser($this->sourceRow);
                    $obj->setNew(false);
                    $this->prefetchedObjects['ozone_user'] = $obj;
                    return $obj;
                }
            }
        }
        return OzoneUserPeer::instance()->selectByPrimaryKey($this->getUserId());
    }

    public function getUserOrString()
    {
        $user = $this->getUser();
        if ($user == null) {
            return $this->getUserString();
        } else {
            return $user;
        }
    }

    public function getOzoneUser()
    {
        return $this->getUser();
    }

    public function getLastPost()
    {
        if ($this->getLastPostId() == null) {
            return;
        }
        $c = new Criteria();
        $c->add("post_id", $this->getLastPostId());
        $c->addJoin("user_id", "ozone_user.user_id");

        $post = ForumPostPeer::instance()->selectOne($c);
        return $post;
    }

    /**
     * Scans for the last post.
     */
    public function findLastPost()
    {
        $c = new Criteria();
        $c->add("thread_id", $this->getThreadId());
        $c->addOrderDescending("post_id");
        $post = ForumPostPeer::instance()->selectOne($c);
        if ($post) {
            $this->setLastPostId($post->getPostId());
        }
        return $post;
    }

    public function calculateNumberPosts()
    {
        $c = new Criteria();
        $c->add("thread_id", $this->getThreadId());
        $num = ForumPostPeer::instance()->selectCount($c);
        $this->setNumberPosts($num);
    }

    public function getCategory()
    {
        $categoryId = $this->getCategoryId();

        $category = ForumCategoryPeer::instance()->selectByPrimaryKey($categoryId);
        return $category;
    }

    public function getForumCategory()
    {
        if (is_array($this->prefetched)) {
            if (in_array('forum_category', $this->prefetched)) {
                if (in_array('forum_thread', $this->prefetchedObjects)) {
                    return $this->prefetchedObjects['forum_category'];
                } else {
                    $obj = new ForumCategory($this->sourceRow);
                    $obj->setNew(false);
                    $this->prefetchedObjects['forum_category'] = $obj;
                    return $obj;
                }
            }
        }
        return ForumCategoryPeer::instance()->selectByPrimaryKey($this->getCategoryId());
    }

    public function getFirstPost()
    {
        $c = new Criteria();
        $c->add("thread_id", $this->getThreadId());
        $c->addOrderAscending("post_id");
        $post = ForumPostPeer::instance()->selectOne($c);
        return $post;
    }

    public function getSite()
    {
        return SitePeer::instance()->selectByPrimaryKey($this->getSiteId());
    }
/*
    public function save(){
        $o = new Outdater();
        $o->forumEvent("thread_save", $this);
        parent::save();
    }
*/
}
