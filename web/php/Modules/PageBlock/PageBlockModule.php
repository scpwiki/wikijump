<?php

namespace Wikidot\Modules\PageBlock;

use Ozone\Framework\Database\Criteria;
use Wikidot\DB\PagePeer;
use Wikidot\DB\ModeratorPeer;
use Wikidot\DB\AdminPeer;

use Ozone\Framework\SmartyModule;
use Wikidot\Utils\ProcessException;
use Wikidot\Utils\WDPermissionException;
use Wikijump\Models\User;

class PageBlockModule extends SmartyModule
{

    public function build($runData)
    {

        $pl = $runData->getParameterList();
        $site = $runData->getTemp("site");

        $pageId = $pl->getParameterValue("page_id");
        $user = $runData->getUser();

        $page = PagePeer::instance()->selectByPrimaryKey($pageId);
        if (!$pageId || $page == null || $page->getSiteId() != $runData->getTemp("site")->getSiteId()) {
            throw new ProcessException(_("Error getting page information."), "no_page");
        }

        if ($this->canSetBlock($user, $page) == false) {
            throw new WDPermissionException(_("Sorry, only Site Admnistrators and selected Moderators can block a page."));
        }

        $runData->contextAdd("page", $page);
    }

    private function canSetBlock($user, $page)
    {

        if ($user->id === User::ADMIN_USER) {
            return true;
        }

        if (!$user) {
            return false;
        }

        // still nothing. check if moderator of "pages".
        $c = new Criteria();
        $c->add("site_id", $page->getSiteId());
        $c->add("user_id", $user->id);
        $rel = ModeratorPeer::instance()->selectOne($c);
        if ($rel && strpos($rel->getPermissions(), 'p') !== false) {
            return true;
        }

        // still nothing. check if admin.
        $c = new Criteria();
        $c->add("site_id", $page->getSiteId());
        $c->add("user_id", $user->id);
        $rel = AdminPeer::instance()->selectOne($c);
        if ($rel) {
            return true;
        }

        return false;
    }
}
