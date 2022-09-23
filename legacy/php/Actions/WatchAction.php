<?php

namespace Wikidot\Actions;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Database\Database;
use Ozone\Framework\SmartyAction;
use Wikidot\DB\WatchedForumThreadPeer;
use Wikidot\DB\WatchedForumThread;
use Wikidot\DB\WatchedPagePeer;
use Wikidot\DB\WatchedPage;
use Wikidot\Utils\ProcessException;
use Wikidot\Utils\WDPermissionException;

class WatchAction extends SmartyAction
{

    public function isAllowed($runData)
    {
        $userId = $runData->getUserId();
        if(!$userId) {
            throw new WDPermissionException(_("This option is available only to registered (and logged-in) users."));
        }
        return true;
    }

    public function perform($r)
    {
    }

    public function watchThreadEvent($runData)
    {
        $pl = $runData->getParameterList();

        $threadId = $pl->getParameterValue('threadId');
        if ($threadId === null || !is_numeric($threadId)) {
            throw new ProcessException(_("Error selecting thread."), "no_thread");
        }

        $user = $runData->getUser();
        if ($user == null) {
            throw new WDPermissionException(_("Sorry, you must be logged in to add thread to watched."));
        }

        $db = Database::connection();
        $db->begin();

        // check if you watch it already

        $c = new Criteria();
        $c->add("user_id", $user->id);
        $c->add("thread_id", $threadId);

        $t = WatchedForumThreadPeer::instance()->selectOne($c);

        if ($t) {
            throw new ProcessException(_("It seems you already watch this thread."), "already_watching");
        }

        // ok, check how many do you already watch. 10 max
        $c = new Criteria();
        $c->add("user_id", $user->id);

        $count = WatchedForumThreadPeer::instance()->selectCount($c);
        if ($count>9) {
            throw new ProcessException(_("You cannot watch more than 10 threads for now."), "max_reached");
        }

        // ok, create new watch.

        $watch = new WatchedForumThread();
        $watch->setUserId($user->id);
        $watch->setThreadId($threadId);

        $watch->save();

        $db->commit();
    }

    public function removeWatchedThreadEvent($runData)
    {
        $pl = $runData->getParameterList();

        $threadId = $pl->getParameterValue("threadId");

        if ($threadId === null || !is_numeric($threadId)) {
            throw new ProcessException(_("Cannot process your request."));
        }

        $c = new Criteria();
        $c->add("thread_id", $threadId);
        $c->add("user_id", $runData->getUserId());

        WatchedForumThreadPeer::instance()->delete($c);
    }

    public function watchPageEvent($runData)
    {
        $pl = $runData->getParameterList();

        $pageId = $pl->getParameterValue('pageId');
        if ($pageId === null || !is_numeric($pageId)) {
            throw new ProcessException(_("Error selecting the page."), "no_page");
        }

        $user = $runData->getUser();
        if ($user == null) {
            throw new WDPermissionException(_("Sorry, you must be logged in to add the page to watched."));
        }

        $db = Database::connection();
        $db->begin();

        // check if you watch it already

        $c = new Criteria();
        $c->add("user_id", $user->id);
        $c->add("page_id", $pageId);

        $t = WatchedPagePeer::instance()->selectOne($c);

        if ($t) {
            throw new ProcessException(_("It seems you already watch this page."), "already_watching");
        }

        // ok, check how many do you already watch. 10 max
        $c = new Criteria();
        $c->add("user_id", $user->id);

        $count = WatchedPagePeer::instance()->selectCount($c);
        if ($count>9) {
            throw new ProcessException(_("You cannot watch more than 10 pages for now."), "max_reached");
        }

        // ok, create new watch.

        $watch = new WatchedPage();
        $watch->setUserId($user->id);
        $watch->setPageId($pageId);

        $watch->save();

        $db->commit();
    }

    public function removeWatchedPageEvent($runData)
    {
        $pl = $runData->getParameterList();

        $pageId = $pl->getParameterValue("pageId");

        if ($pageId === null || !is_numeric($pageId)) {
            throw new ProcessException(_("Cannot process your request."));
        }

        $c = new Criteria();
        $c->add("page_id", $pageId);
        $c->add("user_id", $runData->getUserId());

        WatchedPagePeer::instance()->delete($c);
    }
}
