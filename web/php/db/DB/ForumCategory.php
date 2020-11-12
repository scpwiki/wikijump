<?php
namespace DB;

use \WDStringUtils;
use Database;
use Criteria;

/**
 * Object Model class.
 *
 */
class ForumCategory extends ForumCategoryBase
{

    public function getUnixifiedName()
    {
        return WDStringUtils::toUnixName($this->getName());
    }

    public function getEffectiveMaxNestLevel()
    {
        $nest = $this->getMaxNestLevel();
        if ($nest == null) {
            // get the value from forum settings
            $settings = ForumSettingsPeer::instance()->selectByPrimaryKey($this->getSiteId());
            $nest = $settings->getMaxNestLevel();
        }
        return $nest;
    }

    public function calculateNumberPosts()
    {
        $q = "SELECT sum(number_posts) as posts FROM forum_thread WHERE category_id='".db_escape_string($this->getCategoryId())."'";
        $db = Database::connection();
        $r = $db->query($q);
        $row = $r->nextRow();
        $n = $row['posts'];
        if ($n === null) {
            $n = 0;
        }
        $this->setNumberPosts($n);
    }

    public function calculateNumberThreads()
    {
        $c = new Criteria();
        $c->add("category_id", $this->getCategoryId());
        $num = ForumThreadPeer::instance()->selectCount($c);
        $this->setNumberThreads($num);
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
        $c->add("category_id", $this->getCategoryId());
        $c->add("last_post_id", null, "!=");
        $c->addOrderDescending("last_post_id");
        $thread = ForumThreadPeer::instance()->selectOne($c);
        if ($thread) {
            $this->setLastPostId($thread->getLastPostId());
        } else {
            $this->setLastPostId(null);
        }
    }

    public function getPermissionString()
    {
        if ($this->getPermissions() == null || $this->getPermissions() == '') {
            $settings = ForumSettingsPeer::instance()->selectByPrimaryKey($this->getSiteId());
            return $settings->getPermissions();
        } else {
            return $this->getPermissions();
        }
    }

    public function getForumGroup()
    {
        return ForumGroupPeer::instance()->selectByPrimaryKey($this->getGroupId());
    }
}
