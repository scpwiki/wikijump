<?php

namespace Wikidot\Actions;

use Exception;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Database\Database;
use Ozone\Framework\ODate;
use Ozone\Framework\SmartyAction;
use Wikidot\DB\ForumCategoryPeer;
use Wikidot\DB\ForumThread;
use Wikidot\DB\ForumPostRevision;
use Wikidot\DB\ForumPost;
use Wikidot\DB\ForumThreadPeer;
use Wikidot\DB\ModeratorPeer;
use Wikidot\DB\AdminPeer;
use Wikidot\DB\ForumPostPeer;
use Wikidot\DB\PagePeer;
use Wikidot\DB\ForumCategory;
use Wikidot\DB\ForumGroupPeer;
use Wikidot\DB\ForumGroup;
use Wikidot\Utils\EventLogger;
use Wikidot\Utils\GlobalProperties;
use Wikidot\Utils\Indexer;
use Wikidot\Utils\Outdater;
use Wikidot\Utils\ProcessException;
use Wikidot\Utils\WDPermissionException;
use Wikidot\Utils\WDPermissionManager;
use Wikijump\Models\User;
use Wikijump\Services\Wikitext\ParseRenderMode;
use Wikijump\Services\Wikitext\WikitextBackend;

class ForumAction extends SmartyAction
{

    public function perform($r)
    {
    }

    public function newThreadEvent($runData)
    {
        $pl = $runData->getParameterList();
        $site = $runData->getTemp("site");

        $title = trim($pl->getParameterValue("title"));
        $description = trim($pl->getParameterValue("description"));
        $source = trim($pl->getParameterValue("source"));
        $categoryId = $pl->getParameterValue("category_id");

        $userId = $runData->getUserId();
        if ($userId == null) {
            $userString = $runData->createIpString();
        }

        // validate
        $errors = array();
        if ($title == '') {
            $errors['title'] = _("Thread title cannot be empty.");
        }
        if (strlen8($title)>128) {
            $errors['title'] = _("Thread title should not be longer than 128 characters.");
        }
        if (strlen8($description)>1000) {
            $errors['description'] = _("Thread description should not be longer than 1000 characters.");
        }
        if (strlen($source)>200000) {
            $errors['source'] = _("It seems the source is too long.");
        } elseif ($source == '') {
            $errors['source'] = _("Post body cannot be empty.");
        }
        if (count($errors)>0) {
            $runData->ajaxResponseAdd("formErrors", $errors);
            throw new ProcessException("Form errors", "form_errors");
        }

        // compile content

        $wt = WikitextBackend::make(ParseRenderMode::FORUM_POST, null);
        $body = $wt->renderHtml($source)->body;

        // new thread

        $db = Database::connection();
        $db->begin();

        $c = new Criteria();
        $c->add("category_id", $categoryId);
        $c->setForUpdate(true);
        $category = ForumCategoryPeer::instance()->selectOne($c);
        if ($category == null || $category->getSiteId() !== $site->getSiteId()) {
            throw new ProcessException(_("Problem while selecting forum category."), "no_category");
        }

        WDPermissionManager::instance()->hasForumPermission('new_thread', $runData->getUser(), $category);

        $thread = new ForumThread();
        $thread->setSiteId($site->getSiteId());
        $thread->setCategoryId($categoryId);
        $thread->setTitle($title);
        $thread->setDescription($description);
        $thread->setDateStarted(new ODate());
        $thread->setNumberPosts(1);

        // now set user_id, user_string
        if ($userId) {
            $thread->setUserId($userId);
        } else {
            $thread->setUserId(User::ANONYMOUS_USER);
            $thread->setUserString($userString);
        }

        $thread->save();

        $postRevision = new ForumPostRevision();
        $postRevision->obtainPK();

        $post = new ForumPost();
        $post->obtainPK();

        $postRevision->setPostId($post->getPostId());
        $postRevision->setText($source);
        $postRevision->setTitle($title);
        $postRevision->setDate(new ODate());

        $post->setSiteId($site->getSiteId());
        $post->setRevisionId($postRevision->getRevisionId());
        $post->setText($body);
        $post->setTitle($title);
        $post->setDatePosted(new ODate());
        $post->setThreadId($thread->getThreadId());

        // now set user_id, user_string
        if ($userId) {
            $postRevision->setUserId($userId);
            $post->setUserId($userId);
        } else {
            $postRevision->setUserId(User::ANONYMOUS_USER);
            $post->setUserId(User::ANONYMOUS_USER);
            $postRevision->setUserString($userString);
            $post->setUserString($userString);
        }

        $postRevision->save();
        $post->save();
        $thread->setLastPostId($post->getPostId());
        $thread->save();

        // update number of posts in the category

        $category->setNumberPosts($category->getNumberPosts()+1);
        $category->setNumberThreads($category->getNumberThreads()+1);
        $category->setLastPostId($post->getPostId());
        $category->save();

        $o = new Outdater();
        $o->forumEvent("post_save", $post);

        // index thread
        Indexer::instance()->indexThread($thread);

        $runData->ajaxResponseAdd("threadId", $thread->getThreadId());
        $runData->ajaxResponseAdd("threadUnixifiedTitle", $thread->getUnixifiedTitle());

        EventLogger::instance()->logNewThread($thread);

        $db->commit();
        if (GlobalProperties::$UI_SLEEP) {
            sleep(1);
        }
    }

    public function savePostEvent($runData)
    {
        $pl = $runData->getParameterList();
        $site = $runData->getTemp("site");

        $title = trim($pl->getParameterValue("title"));
        $source = trim($pl->getParameterValue("source"));
        $threadId = $pl->getParameterValue("threadId");
        $parentPostId = $pl->getParameterValue("parentId");
        $user = $runData->getUser();
        $userId = $user->id;
        if ($user == null) {
            $userString = $runData->createIpString();
        }

        $errors = [];
        if (strlen8($title)>128) {
            $errors['title'] = _("Post title should not be longer than 128 characters.");
        }

        if (strlen($source)>200000) {
            $errors['source'] = _("It seems the source is too long.");
        } elseif ($source == '') {
            $errors['source'] = _("Post body cannot be empty.");
        }
        if (count($errors)>0) {
            $runData->ajaxResponseAdd("formErrors", $errors);
            throw new ProcessException("Form errors", "form_errors");
        }

        $c = new Criteria();
        $c->add("thread_id", $threadId);
        $c->add("site_id", $site->getSiteId());
        $c->setForUpdate(true);
        $thread = ForumThreadPeer::instance()->selectOne($c);

        if ($thread == null || $thread->getSiteId() != $site->getSiteId()) {
            throw new ProcessException(_("Thread not found."), "no_thread");
        }

        if ($thread->getBlocked()) {
            // check if moderator or admin
            $c = new Criteria();
            $c->add("site_id", $site->getSiteId());
            $c->add("user_id", $user->id);
            $rel = ModeratorPeer::instance()->selectOne($c);
            if (!$rel || strpos($rel->getPermissions(), 'f') == false) {
                $rel = AdminPeer::instance()->selectOne($c);
                if (!$rel) {
                    throw new WDPermissionException(_("Sorry, this thread is blocked. Nobody can add new posts nor edit existing ones."));
                }
            }
        }

        // compile content

        $wt = WikitextBackend::make(ParseRenderMode::FORUM_POST, null);
        $body = $wt->renderHtml($source)->body;

        $db = Database::connection();
        $db->begin();

        $c = new Criteria();
        $c->add("category_id", $thread->getCategoryId());
        $c->setForUpdate(true);
        $category = ForumCategoryPeer::instance()->selectOne($c);
        if ($category == null || $category->getSiteId() !== $site->getSiteId()) {
            throw new ProcessException(_("Problem while selecting forum category."), "no_category");
        }
        WDPermissionManager::instance()->hasForumPermission('new_post', $runData->getUser(), $category);

        $postRevision = new ForumPostRevision();
        $postRevision->obtainPK();

        $post = new ForumPost();
        $post->obtainPK();

        $postRevision->setPostId($post->getPostId());
        $postRevision->setText($source);
        $postRevision->setTitle($title);
        $postRevision->setDate(new ODate());

        $post->setSiteId($site->getSiteId());
        $post->setRevisionId($postRevision->getRevisionId());
        $post->setText($body);
        $post->setTitle($title);
        $post->setDatePosted(new ODate());
        $post->setThreadId($threadId);

        if ($parentPostId) {
            $post->setParentId($parentPostId);
        }

        // now set user_id, user_string
        if ($userId) {
            $postRevision->setUserId($userId);
            $post->setUserId($userId);
        } else {
            $postRevision->setUserId(User::ANONYMOUS_USER);
            $post->setUserId(User::ANONYMOUS_USER);
            $postRevision->setUserString($userString);
            $post->setUserString($userString);
        }

        $postRevision->save();
        $post->save();

        $thread->setLastPostId($post->getPostId());
        $thread->setNumberPosts($thread->getNumberPosts()+1);
        $thread->save();

        // update number of posts in the category

        $category->setNumberPosts($category->getNumberPosts()+1);

        $category->setLastPostId($post->getPostId());
        $category->save();

        $o = new Outdater();
        $o->forumEvent("post_save", $post);

        // index thread
        Indexer::instance()->indexThread($thread);

        EventLogger::instance()->logNewPost($post);

        $db->commit();

        $runData->ajaxResponseAdd("postId", $post->getPostId());
        if (GlobalProperties::$UI_SLEEP) {
            sleep(1);
        }
    }

    public function saveEditPostEvent($runData)
    {
        $pl = $runData->getParameterList();
        $site = $runData->getTemp("site");

        $title = $pl->getParameterValue("title");
        $source = $pl->getParameterValue("source");
        $threadId = $pl->getParameterValue("threadId");
        $postId = $pl->getParameterValue("postId");
        $currentRevisionId = $pl->getParameterValue("currentRevisionId");

        $user = $runData->getUser();

        $userId = $user->id;
        if ($user == null) {
            $userString = $runData->createIpString();
        }

        $errors = [];
        if (strlen8($title)>128) {
            $errors['title'] = _("Post title should not be longer than 128 characters.");
        }

        if (strlen($source)>200000) {
            $errors['source'] = _("It seems the source is too long.");
        } elseif ($source == '') {
            $errors['source'] = _("Post body cannot be empty.");
        }
        if (count($errors)>0) {
            $runData->ajaxResponseAdd("formErrors", $errors);
            throw new ProcessException("Form errors", "form_errors");
        }

        $db = Database::connection();
        $db->begin();

        if ($postId == null || !is_numeric($postId)) {
            throw new ProcessException(_("No such post."), "no_post");
        }

        $post = ForumPostPeer::instance()->selectByPrimaryKey($postId);
        if ($post == null || $post->getSiteId() != $site->getSiteId()) {
            throw new ProcessException(_("No such post."), "no_post");
        }

        $c = new Criteria();
        $c->add("thread_id", $post->getThreadId());
        $c->add("site_id", $site->getSiteId());
        $c->setForUpdate(true);
        $thread = ForumThreadPeer::instance()->selectOne($c);

        if ($thread == null || $thread->getSiteId() != $site->getSiteId()) {
            throw new ProcessException("Thread not found.", "no_thread");
        }

        // check revisions...
        if ($post->getRevisionId() != $currentRevisionId) {
            throw new ProcessException(_("The post has been changed meanwhile. Sorry, you cannot save it. Please reload the page and start editing again with the current revision."), "no_post");
        }

        $category = $post->getForumThread()->getCategory();
        WDPermissionManager::instance()->hasForumPermission('edit_post', $runData->getUser(), $category, null, $post);

        if ($thread->getBlocked()) {
            // check if moderator or admin
            $c = new Criteria();
            $c->add("site_id", $site->getSiteId());
            $c->add("user_id", $user->id);
            $rel = ModeratorPeer::instance()->selectOne($c);
            if (!$rel || strpos($rel->getPermissions(), 'f') == false) {
                $rel = AdminPeer::instance()->selectOne($c);
                if (!$rel) {
                    throw new WDPermissionException(_("Sorry, this thread is blocked. Nobody can  add new posts nor edit existing ones."));
                }
            }
        }

        // compile content

        $wt = WikitextBackend::make(ParseRenderMode::FORUM_POST, null);
        $body = $wt->renderHtml($source)->body;

        $postRevision = new ForumPostRevision();
        $postRevision->obtainPK();
        $postRevision->setPostId($post->getPostId());
        $postRevision->setText($source);
        $postRevision->setTitle($title);
        $postRevision->setDate(new ODate());

        $post->setRevisionId($postRevision->getRevisionId());
        $post->setRevisionNumber($post->getRevisionNumber()+1);
        $post->setText($body);
        $post->setTitle($title);
        $post->setDateLastEdited(new ODate());
        if ($userId) {
            $postRevision->setUserId($userId);
            $post->setEditedUserId($userId);
        } else {
            $postRevision->setUserId(User::ANONYMOUS_USER);
            $post->setEditedUserId(User::ANONYMOUS_USER);
            $postRevision->setUserString($userString);
            $post->setEditedUserString($userString);
        }
        $post->save();
        $postRevision->save();

        $o = new Outdater();
        $o->forumEvent("post_save", $post);

        // index thread
        Indexer::instance()->indexThread($thread);

        EventLogger::instance()->logSavePost($post);

        $db->commit();
        $runData->ajaxResponseAdd("postId", $post->getPostId());

        if (GlobalProperties::$UI_SLEEP) {
            sleep(1);
        }
    }

    public function createPageDiscussionThreadEvent($runData)
    {
        // ok, the page has no discussion yet... but check it!
        $site = $runData->getTemp("site");
        $pl = $runData->getParameterList();
        $pageId = $pl->getParameterValue("page_id");

        $page = PagePeer::instance()->selectByPrimaryKey($pageId);
        if ($page == null || $page->getSiteId() != $site->getSiteId()) {
            throw new ProcessException(_("Page does not exist."), "no_page");
        }

        $db = Database::connection();
        $db->begin();

        $c = new Criteria();
        $c->add("page_id", $pageId);
        $c->add("site_id", $site->getSiteId());

        $thread = ForumThreadPeer::instance()->selectOne($c);
        if ($thread) {
            // thread exists! which means it could have been created meanwhile!
            // simply return the thread it now.
            $runData->ajaxResponseAdd("thread_id", $thread->getThreadId());
            $runData->ajaxResponseAdd("thread_unix_title", $thread->getUnixifiedTitle());
            $db->commit();
            return;
        }

        // thread does not exist. check if category with page discussions exist.
        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        $c->add("per_page_discussion", true);

        $category = ForumCategoryPeer::instance()->selectOne($c);

        if ($category == null) {
            // create this category!
            $category = new ForumCategory();
            $category->setName("Per page discussions");
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
                $group->setName("Hidden");
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
        $thread->setPageId($pageId);
        $thread->setUserId(User::AUTOMATIC_USER);
        $thread->setDateStarted(new ODate());
        $thread->setNumberPosts(0);
        $thread->save();

        $page->setThreadId($thread->getThreadId());
        $page->save();

        $category->setNumberThreads($category->getNumberThreads()+1);
        $category->save();

        $runData->ajaxResponseAdd("thread_id", $thread->getThreadId());
        $runData->ajaxResponseAdd("thread_unix_title", $thread->getUnixifiedTitle());

        $o = new Outdater();
        $o->forumEvent("thread_save", $thread);

        $db->commit();
    }

    public function saveThreadMetaEvent($runData)
    {
        $pl = $runData->getParameterList();

        $threadId = $pl->getParameterValue("threadId");
        $site = $runData->getTemp("site");

        $title = $pl->getParameterValue("title");
        $description = $pl->getParameterValue("description");

        $user = $runData->getUser();

        // validate
        $errors = array();
        if ($title == '') {
            $errors['title'] = _("Thread title cannot be empty.");
        }
        if (strlen8($title)>128) {
            $errors['title'] = _("Thread title should not be longer than 128 characters.");
        }
        if (strlen($description)>1000) {
            $errors['description'] = _("Thread description should not be longer than 1000 characters.");
        }
        if (count($errors)>0) {
            $runData->ajaxResponseAdd("formErrors", $errors);
            throw new ProcessException("Form errors", "form_errors");
        }
        $db = Database::connection();
        $db->begin();

        $thread = ForumThreadPeer::instance()->selectByPrimaryKey($threadId);
        if ($thread == null || $thread->getSiteId() !== $site->getSiteId()) {
            throw new ProcessException(_("No thread found... Is it deleted?"), "no_thread");
        }

        if ($thread->getBlocked()) {
            // check if moderator or admin
            $c = new Criteria();
            $c->add("site_id", $site->getSiteId());
            $c->add("user_id", $user->id);
            $rel = ModeratorPeer::instance()->selectOne($c);
            if (!$rel || strpos($rel->getPermissions(), 'f') == false) {
                $rel = AdminPeer::instance()->selectOne($c);
                if (!$rel) {
                    throw new WDPermissionException(_("Sorry, this thread is blocked. Meta information cannot be edited."));
                }
            }
        }

        $category = $thread->getCategory();
        WDPermissionManager::instance()->hasForumPermission('edit_thread', $runData->getUser(), $category, $thread);

        $changed = false;
        $title = trim($title);
        $description = trim($description);

        if ($title !== $thread->getTitle()) {
            $changed = true;
            $thread->setTitle($title);
        }
        if ($description !== $thread->getDescription()) {
            $changed = true;
            $thread->setDescription($description);
        }
        if ($changed) {
            $thread->save();
            EventLogger::instance()->logSaveThreadMeta($thread);
        }

        $o = new Outdater();
        $o->forumEvent("thread_save", $thread);

        // index thread
        Indexer::instance()->indexThread($thread);

        $db->commit();
        if (GlobalProperties::$UI_SLEEP) {
            sleep(1);
        }
    }

    public function saveStickyEvent($runData)
    {
        $pl = $runData->getParameterList();

        $threadId = $pl->getParameterValue("threadId");
        $site = $runData->getTemp("site");

        $sticky = $pl->getParameterValue("sticky");

        $db = Database::connection();
        $db->begin();

        $thread = ForumThreadPeer::instance()->selectByPrimaryKey($threadId);
        if ($thread == null || $thread->getSiteId() !== $site->getSiteId()) {
            throw new ProcessException(_("No thread found... Is it deleted?"), "no_thread");
        }
        $category = $thread->getForumCategory();
        WDPermissionManager::instance()->hasForumPermission('moderate_forum', $runData->getUser(), $category);

        if ($sticky) {
            $thread->setSticky(true);
        } else {
            $thread->setSticky(false);
        }
        $thread->save();

        $o = new Outdater();
        $o->forumEvent("thread_save", $thread);

        EventLogger::instance()->logSaveThreadStickness($thread);

        $db->commit();
        if (GlobalProperties::$UI_SLEEP) {
            sleep(1);
        }
    }

    public function saveBlockEvent($runData)
    {
        $pl = $runData->getParameterList();

        $threadId = $pl->getParameterValue("threadId");
        $site = $runData->getTemp("site");

        $block = $pl->getParameterValue("block");

        $db = Database::connection();
        $db->begin();

        $thread = ForumThreadPeer::instance()->selectByPrimaryKey($threadId);
        if ($thread == null || $thread->getSiteId() !== $site->getSiteId()) {
            throw new ProcessException(_("No thread found... Is it deleted?"), "no_thread");
        }
        $category = $thread->getForumCategory();
        WDPermissionManager::instance()->hasForumPermission('moderate_forum', $runData->getUser(), $category);

        if ($block) {
            $thread->setBlocked(true);
        } else {
            $thread->setBlocked(false);
        }
        $thread->save();
        EventLogger::instance()->logSaveThreadBlock($thread);

        $o = new Outdater();
        $o->forumEvent("thread_save", $thread);

        $db->commit();
        if (GlobalProperties::$UI_SLEEP) {
            sleep(1);
        }
    }

    public function moveThreadEvent($runData)
    {
        $pl = $runData->getParameterList();
        $threadId = $pl->getParameterValue("threadId");
        $site = $runData->getTemp("site");

        $categoryId = $pl->getParameterValue("categoryId");

        $db = Database::connection();
        $db->begin();

        $thread = ForumThreadPeer::instance()->selectByPrimaryKey($threadId);
        if ($thread == null || $thread->getSiteId() !== $site->getSiteId()) {
            throw new ProcessException(_("No thread found... Is it deleted?"), "no_thread");
        }

        if ($thread->getCategoryId() == $categoryId) {
            throw new ProcessException(_("Destination category is the same as current. Not moved."), "same_category");
        }

        $oldCategory = $thread->getForumCategory();

        // get destination category
        $category = ForumCategoryPeer::instance()->selectByPrimaryKey($categoryId);
        if ($category == null || $category->getSiteId() !== $site->getSiteId()) {
            throw new ProcessException(_("No destination category found... Is it deleted?"), "no_thread");
        }

        WDPermissionManager::instance()->hasForumPermission('moderate_forum', $runData->getUser(), $category);

        $thread->setCategoryId($categoryId);
        $thread->save();

        $oldCategory->calculateNumberThreads();
        $oldCategory->calculateNumberPosts();
        $oldCategory->findLastPost();
        $oldCategory->save();

        $category->calculateNumberThreads();
        $category->calculateNumberPosts();
        $category->findLastPost();
        $category->save();

        $o = new Outdater();
        $o->forumEvent("outdate_forum");

        // index thread
        Indexer::instance()->indexThread($thread);

        EventLogger::instance()->logThreadMoved($thread, $category);

        $db->commit();
        if (GlobalProperties::$UI_SLEEP) {
            sleep(1);
        }
    }

    public function deletePostEvent($runData)
    {

        $pl = $runData->getParameterList();
        $site = $runData->getTemp("site");

        $postId = $pl->getParameterValue("postId");

        if ($postId == null || !is_numeric($postId)) {
            throw new ProcessException(_("No such post."), "no_post");
        }

        $db = Database::connection();
        $db->begin();

        $post = ForumPostPeer::instance()->selectByPrimaryKey($postId);
        if ($post == null || $post->getSiteId() != $site->getSiteId()) {
            throw new ProcessException(_("No such post."), "no_post");
        }

        $thread = $post->getForumThread();
        $category = $thread->getForumCategory();

        try {
            WDPermissionManager::instance()->hasForumPermission('moderate_forum', $runData->getUser(), $category);
        } catch (Exception $e) {
            throw new WDPermissionException(_("Sorry, you are not allowed to delete posts. Only site administrators and moderators are the ones who can."));
        }

        $c = new Criteria();
        $c->add("parent_id", $postId);
        $toDelete = array();

        $chposts =  ForumPostPeer::instance()->select($c);

        while ($chposts && count($chposts) >0) {
            $toDelete = array_merge($toDelete, $chposts);

            $c = new Criteria();
            foreach ($chposts as $f) {
                $c->addOr("parent_id", $f->getPostId());
            }
            $chposts =  ForumPostPeer::instance()->select($c);
        }

        ForumPostPeer::instance()->deleteByPrimaryKey($post->getPostId());
        foreach ($toDelete as $f) {
            ForumPostPeer::instance()->deleteByPrimaryKey($f->getPostId());
        }

        // now recalculate a few things...
        $thread->calculateNumberPosts();
        $thread->findLastPost();
        $thread->save();

        $category->calculateNumberPosts();
        $category->findLastPost();
        $category->save();

        // outdate
        $o = new Outdater();
        $o->forumEvent("thread_save", $thread);

        // index thread
        Indexer::instance()->indexThread($thread);

        EventLogger::instance()->logPostDelete($thread, $post->getTitle());

        $db->commit();
        if (GlobalProperties::$UI_SLEEP) {
            sleep(1);
        }
    }
}
