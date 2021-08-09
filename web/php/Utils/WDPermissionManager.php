<?php

namespace Wikidot\Utils;


use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Database\Database;
use Ozone\Framework\Ozone;
use Wikidot\DB\Site;
use Wikidot\DB\SitePeer;
use Wikidot\DB\AdminPeer;
use Wikidot\DB\ModeratorPeer;
use Wikidot\DB\MemberPeer;
use Wikidot\DB\Page;
use Wikidot\DB\PagePeer;
use Wikidot\DB\IpBlockPeer;
use Wikidot\DB\UserBlockPeer;
use Wikijump\Models\User;

class WDPermissionManager
{

    private static $instance;

    private $checkIpBlocks = true;
    private $checkUserBlocks = true;
    /**
     * If set to true each permission check will generate a
     * PermissionException instead of returning false if
     * action is not permitted.
     */
    private $throwExceptions = true;

    private static $pageActions = array(
            'view' => 'v',
            'edit' => 'e',
            'create' => 'c',
            'move' => 'm',
            'delete' => 'd',
            'attach_file' => 'a',
            'rename_file' => 'r',
            'replace_file' => 'z',
            'move_file' => 'z',
            'delete_file' => 'z',
            'options' => 'o'
        );

    private static $pageActionsDesc = array(
            'view' => 'view this page',
            'edit' => 'edit this page',
            'create' => 'create a new page in this category',
            'move' => 'move this page',
            'delete' => 'delete this page',
            'attach_file' => 'attach a new file to this page',
            'rename_file' => 'rename file attachment in this page',
            'replace_file' => 'replace existing file attachment in this page',
            'move_file' => 'move file attachment to another page',
            'delete_file' => 'delete file in this page',
            'options' => 'what????'
        );

    private static $forumActions = array(
            'new_thread' => 't',
            'new_post' => 'p',
            'edit_post' => 'e',
            'edit_thread' => 'e',
            'split' => 's',
            'moderate_forum' => 'x'
        );
    private static $forumActionsDesc = array(
            'new_thread' => 'start new discussion thread',
            'new_post' => 'add new post in this thread',
            'edit_post' => 'edit a post in this thread',
            'edit_thread' => 'edit this thread',
            'split' => 's',
            'moderate_forum' => 'perform this action'
        );

    private static $userClasses = array(
        'anonymous' => 'a',
        'registered' => 'r',
        'member' => 'm',
        'owner' => 'o'
    );

    private static $userClassesDesc = array(
        'anonymous' => 'anonymous users',
        'registered' => 'registered users',
        'member' => 'members of this site',
        'owner' => 'owner (creator) of this page'
    );

    public static function instance()
    {
        if (self::$instance == null) {
            self::$instance = new WDPermissionManager();
        }
        return self::$instance;
    }


    public function __construct()
    {

        self::$pageActionsDesc = array(
            'view' => _('view this page'),
            'edit' => _('edit this page'),
            'create' => _('create a new page in this category'),
            'move' => _('move this page'),
            'delete' => _('delete this page'),
            'attach_file' => _('attach a new file to this page'),
            'rename_file' => _('rename file attachment in this page'),
            'replace_file' => _('replace existing file attachment in this page'),
            'move_file' => _('move file attachment to another page'),
            'delete_file' => _('delete file in this page'),
            'options' => _('what????')
        );

        self::$forumActionsDesc = array(
            'new_thread' => _('start new discussion thread'),
            'new_post' => _('add new post in this thread'),
            'edit_post' => _('edit a post in this thread'),
            'edit_thread' => _('edit this thread'),
            'split' => 's',
            'moderate_forum' => _('perform this action'));

        self:: $userClassesDesc = array(
            'anonymous' => _('anonymous users'),
            'registered' => _('registered users'),
            'member' => _('members of this site'),
            'owner' => _('owner (creator) of this page'));
    }

    public function hasPermission($action, $user, $site = null)
    {
         if ($user->id === 1) {
            return true;
        }

        if (($site && is_string($site) && is_numeric($site)) || is_int($site)) {
            $site = SitePeer::instance()->selectByPrimaryKey($site);
        }

        if ($site && $site->getDeleted()) {
            $message = _("This site has been deleted.");
            $action = null;
        }

        switch ($action) {
            /*
             * check for site management permissions.
             * user must be in the admin group.
             */

            case 'manage_site':
                if (!$user) {
                    $message = _("You have no permission to configure the site properties. Only site admins are allowed to do it.".
                    "But right now you are not even logged in...");
                } else {
                    $c = new Criteria();
                    $c->add("user_id", $user->id);
                    $c->add("site_id", $site->getSiteId());
                    $rel = AdminPeer::instance()->selectOne($c);
                    if ($rel == null) {
                        $message = _("You have no permission to configure the site properties. Only site admins are allowed to do it.");
                    }
                }
                break;

            case 'moderate_site':
                if (!$user) {
                    $message = _("You have no permission to moderate the site. Only site moderatorss are allowed to do it.".
                    "But right now you are not even logged in...");
                } else {
                    $c = new Criteria();
                    $c->add("user_id", $user->id);
                    $c->add("site_id", $site->getSiteId());
                    $rel = ModeratorPeer::instance()->selectOne($c);
                    if ($rel == null) {
                        $message = _("You have no permission to moderate the site properties. Only site moderators are allowed to do it.");
                    }
                }
                break;

            case 'account':
                if (!$user) {
                    $message = _("You cannot access your account while not being logged in...");
                }
                break;
            case 'become_member':

                $c = new Criteria();
                ;
                $c->add("user_id", $user->id);

                $mc = MemberPeer::instance()->selectCount($c);

                // count memberships

                // check if not on a blacklist
                $block = $this->checkUserBlocked($user, $site);
                if ($block) {
                    $message = _("Sorry, you are banned from this site and not allowed to join.");
                    if ($block->getReason() && $block->getReason()!='') {
                        $message .= _("The given reason is:")." <p>".htmlspecialchars($block->getReason())."</p>";
                    }
                }
                break;
        }

        if ($message) {
            // shit. no permission.
            if ($this->throwExceptions == true) {
                // throw exception
                throw new WDPermissionException($message);
            } else {
                return false;
            }
        } else {
            return true;
        }
    }


    public function hasPagePermission($action, $user, $category, $page = null, $site = null)
    {
        if ($user->id == 1) {
            return true;
        }

        $site = $GLOBALS['site']; // ugly.

        // ban by IP first.
        if ($this->checkIpBlocks) {
            $ips = Ozone::getRunData()->createIpString();
            $blocks = $this->checkIpBlocked($ips, $site);
            if (count($blocks)>0) {
                if ($this->throwExceptions) {
                    throw new WDPermissionException(_("Sorry, your IP address is blocked from participating in and modifying this site."));
                } else {
                    return false;
                }
            }
        }

        // check if page not blocked
        if ($page && $page instanceof Page && $page->getBlocked()) {
            if ($user) {
                // still nothing. check if moderator of "pages".
                $c = new Criteria();
                $c->add("site_id", $category->getSiteId());
                $c->add("user_id", $user->id);
                $rel = ModeratorPeer::instance()->selectOne($c);
                if ($rel && strpos($rel->getPermissions(), 'p') !== false) {
                    return true;
                }

                // still nothing. check if admin.
                $c = new Criteria();
                $c->add("site_id", $category->getSiteId());
                $c->add("user_id", $user->id);
                $rel = AdminPeer::instance()->selectOne($c);
                if ($rel) {
                    return true;
                }
            }
            // if not - can not edit!
            throw new WDPermissionException(_("This page is blocked and only Site Administrators and Moderators with enough privileges can modify it."));
        }



        //action code
        $ac = self::$pageActions[$action];
        //permission string
        $ps = $category->getPermissionString();

        // first try anonymous and registered to save effort
        $uc = self::$userClasses['anonymous'];
        if ($this->permissionLookup($ac, $uc, $ps)) {
            // ok, anyone can.
            // but check ip blocks.
            if ($this->checkUserBlocks && $user) {
                    $block = $this->checkUserBlocked($user, $site);
                if ($block) {
                    if ($this->throwExceptions) {
                        $message = _("Sorry, you are blocked from participating in and modifying this site. ");
                        if ($block->getReason() && $block->getReason()!='') {
                            $message .= _("The given reason is:")." <p>".htmlspecialchars($block->getReason())."</p>";
                        }
                        throw new WDPermissionException($message);
                    } else {
                        return false;
                    }
                }
                //}
            }
            return true;
        } elseif (!$user) {
            // anonymous can not and the user is only anonymous. game over.

            $m = $this->generateMessage($action, $uc, $ps);
            $this->handleFalse($m);
            return false;
        }

        // ok, check registered now
        $uc = self::$userClasses['registered'];
        if ($this->permissionLookup($ac, $uc, $ps)) {
            // check blocked users
            if ($this->checkUserBlocks) {
                $block = $this->checkUserBlocked($user, $site);
                if ($block) {
                    if ($this->throwExceptions) {
                        $message = _("Sorry, you are blocked from participating in and modifying this site. ");
                        if ($block->getReason() && $block->getReason()!='') {
                            $message .= _("The given reason is:")." <p>".htmlspecialchars($block->getReason())."</p>";
                        }
                        throw new WDPermissionException($message);
                    } else {
                        return false;
                    }
                }
            }
            return true;
        }

        // ok, a "premium feature" or what... need to check members now...
        $uc = self::$userClasses['member'];
        if ($this->permissionLookup($ac, $uc, $ps)) {
            // ok, members CAN do this. is the user a member?
            $c = new Criteria();
            $c->add("site_id", $category->getSiteId());
            $c->add("user_id", $user->id);
            $rel = MemberPeer::instance()->selectOne($c);
            if ($rel) {
                return true;
            }
        }

        $uc = self::$userClasses['owner'];
        if ($page && $this->permissionLookup($ac, $uc, $ps)) {
            if ($site && is_string($page)) {
                $page = PagePeer::instance()->selectByName($site->getSiteId(), $page);
            }
            if ($page && $page->getOwnerUserId() && $user->id == $page->getOwnerUserId()) {
                // check blocked users
                if ($this->checkUserBlocks) {
                    $block = $this->checkUserBlocked($user, $site);
                    if ($block) {
                        if ($this->throwExceptions) {
                            $message = _("" .
                                    "Sorry, you are blocked from participating in and modifying this site. ");
                            if ($block->getReason() && $block->getReason()!='') {
                                $message .= _("The given reason is:")." <p>".htmlspecialchars($block->getReason())."</p>";
                            }
                            throw new WDPermissionException($message);
                            //throw new WDPermissionException("Sorry, you are blocked from participating in and modifying this site. " .
                            //      "The given reason is: \"".htmlspecialchars($block->getReason())."\"");
                        } else {
                            return false;
                        }
                    }
                }
                return true;
            }
        }

        // still nothing. check if moderator of "pages".
        $c = new Criteria();
        $c->add("site_id", $category->getSiteId());
        $c->add("user_id", $user->id);
        $rel = ModeratorPeer::instance()->selectOne($c);
        if ($rel && strpos($rel->getPermissions(), 'p') !== false) {
            return true;
        }

        // still nothing. check if admin.
        $c = new Criteria();
        $c->add("site_id", $category->getSiteId());
        $c->add("user_id", $user->id);
        $rel = AdminPeer::instance()->selectOne($c);
        if ($rel) {
            return true;
        }

        $m = $this->generateMessage($action, $uc, $ps);
        $this->handleFalse($m);
        return false;
    }

    public function hasForumPermission($action, $user, $category, $thread = null, $post = null)
    {
        if ($user->id == 1) {
            return true;
        }

        $site = $GLOBALS['site']; // ugly.

        // ban by IP first.
        if ($this->checkIpBlocks) {
            $ips = Ozone::getRunData()->createIpString();
            $blocks = $this->checkIpBlocked($ips, $site);
            if (count($blocks)>0) {
                if ($this->throwExceptions) {
                    throw new WDPermissionException(_("Sorry, your IP address is blocked from participating in and modifying this site."));
                } else {
                    return false;
                }
            }
        }

        if (strpos($action, "thread")) {
            $authorString = _("author of the thread");
        } else {
            $authorString = _("author of the post");
        }

        //action code
        $ac = self::$forumActions[$action];
        //permission string
        $ps = $category->getPermissionString();

        //throw new WDPermissionException($ps);

        // first try anonymous and registered to save effort
        $uc = self::$userClasses['anonymous'];
        if ($this->permissionLookup($ac, $uc, $ps)) {
            // ok, anyone can.
            // but check ip blocks.
            if ($this->checkUserBlocks && $user) {
                    $block = $this->checkUserBlocked($user, $site);
                if ($block) {
                    if ($this->throwExceptions) {
                        $message = _("Sorry, you are blocked from participating in and modifying this site. ");
                        if ($block->getReason() && $block->getReason()!='') {
                            $message .= _("The given reason is:")." <p>".htmlspecialchars($block->getReason())."</p>";
                        }
                        throw new WDPermissionException($message);
                        //throw new WDPermissionException("Sorry, you are blocked from participating in and modifying this site. " .
                        //      "The given reason is: \"".htmlspecialchars($block->getReason())."\"");
                    } else {
                        return false;
                    }
                }
                //}
            }
            return true;
        } elseif (!$user) {
            // anonymous can not and the user is only anonymous. game over.
        //  throw new WDPermissionException($ps);
            $m = $this->generateMessage($action, $uc, $ps, 'forum', array("o" => $authorString));
            $this->handleFalse($m);
            return false;
        }

        // ok, check registered now
        $uc = self::$userClasses['registered'];
        if ($this->permissionLookup($ac, $uc, $ps)) {
            // check blocked users
            if ($this->checkUserBlocks) {
                $block = $this->checkUserBlocked($user, $site);
                if ($block) {
                    if ($this->throwExceptions) {
                        $message = _("Sorry, you are blocked from participating in and modifying this site. ");
                        if ($block->getReason() && $block->getReason()!='') {
                            $message .= _("The given reason is:")." <p>".htmlspecialchars($block->getReason())."</p>";
                        }
                        throw new WDPermissionException($message);
                        //throw new WDPermissionException("Sorry, you are blocked from participating in and modifying this site. " .
                        //      "The given reason is: \"".htmlspecialchars($block->getReason())."\"");
                    } else {
                        return false;
                    }
                }
            }
            return true;
        }

        // ok, a "premium feature" or what... need to check members now...
        $uc = self::$userClasses['member'];
        if ($this->permissionLookup($ac, $uc, $ps)) {
            // ok, members CAN do this. is the user a member?
            $c = new Criteria();
            $c->add("site_id", $category->getSiteId());
            $c->add("user_id", $user->id);
            $rel = MemberPeer::instance()->selectOne($c);
            if ($rel) {
                return true;
            }
        }

        $uc = self::$userClasses['owner'];
        if (($post || $thread) && $this->permissionLookup($ac, $uc, $ps)) {
            $o = $post ? $post:$thread;

            if ($o && $o->getUserId() && $user->id == $o->getUserId()) {
                // check blocked users
                if ($this->checkUserBlocks) {
                    $block = $this->checkUserBlocked($user, $site);
                    if ($block) {
                        if ($this->throwExceptions) {
                            $message = _("Sorry, you are blocked from participating in and modifying this site. ");
                            if ($block->getReason() && $block->getReason()!='') {
                                $message .= _("The given reason is:")." <p>".htmlspecialchars($block->getReason())."</p>";
                            }
                            throw new WDPermissionException($message);
                            //throw new WDPermissionException("Sorry, you are blocked from participating in and modifying this site. " .
                            //      "The given reason is: \"".htmlspecialchars($block->getReason())."\"");
                        } else {
                            return false;
                        }
                    }
                }
                return true;
            }
        }


        // still nothing. check if moderator of "forum".
        $c = new Criteria();
        $c->add("site_id", $category->getSiteId());
        $c->add("user_id", $user->id);
        $rel = ModeratorPeer::instance()->selectOne($c);
        if ($rel && strpos($rel->getPermissions(), 'f') !== false) {
            return true;
        }

        // still nothing. check if admin.
        $c = new Criteria();
        $c->add("site_id", $category->getSiteId());
        $c->add("user_id", $user->id);
        $rel = AdminPeer::instance()->selectOne($c);
        if ($rel) {
            return true;
        }

        $m = $this->generateMessage($action, $uc, $ps, 'forum', array("o" => $authorString));
        $this->handleFalse($m);
        return false;
    }

    /**
     * Check if $user can send a private message to $toUser.
     */
    public function hasPmPermission(User $user, User $toUser)
    {

        if ($user->id == 1) {
            return true;
        }

        // first check if if $user has pm enabled
        $p = $toUser->get('receive_pm');
        //echo "ad";
        if ($this->isUserSuperior($user, $toUser)) {
            return true;
        }

        // accept from none (unless from moderators/admins of common sites)

        if ($p == 'n') {
            throw new WDPermissionException(_("This user does wish to receive private messages."));
        }

        if ($p == 'mf') {
            if ($this->shareSites($user, $toUser)) {
                // so they share common sites. check for blocks!
                if ($user->isBlockedByUser($toUser)) {
                    throw new WDPermissionException(_("You are blocked by this user."));
                }
                return true;
            }

            // if friends - return true (todo)
            if ($user->isContact($toUser)) {
                return true;
            }

            throw new WDPermissionException(_("This user wishes to receive messages only from selected users."));
        }

        if ($p == 'f') {
            if ($user->isContact($toUser)) {
                return true;
            }
            throw new WDPermissionException(_("This user wishes to receive messages only from selected users."));
        }

        if ($p == 'a') {
            // check if not blocked
            if ($user->isBlockedByUser($toUser)) {
                throw new WDPermissionException(_("You are blocked by this user."));
            }
        }

        // in any other case check
        return true;
    }

    public function canBecomeAdmin($user)
    {

        if ($user->id == 1) {
            return true;
        }

        // check how many sites does the user administer.
        $us = $user->getSettings();
        if ($us->getMaxSitesAdmin()) {
            /* If null, then unlimited. */
            $c = new Criteria();
            $c->add("user_id", $user->id);
            $c->addJoin("site_id", "site.site_id");
            $c->add('founder', false);
            $c->add("site.deleted", false);

            $ac = AdminPeer::instance()->selectCount($c);
            $us = $user->getSettings();
            if ($ac >= $us->getMaxSitesAdmin()) {
                throw new WDPermissionException(sprintf(_("Sorry, you can be a guest administrator of max %d Sites."), $us->getMaxSitesAdmin()));
            }
        }


        return true;
    }

    public function canBecomeMaster($user)
    {
        if ($user->id == 1) {
            return true;
        }
        $us = $user->getSettings();
        if ($us->getMaxSitesMaster()) {
            $c = new Criteria();
            $c->add("user_id", $user->id);
            $c->addJoin("site_id", "site.site_id");
            $c->add('founder', true);
            $c->add("site.deleted", false);

            $ac = AdminPeer::instance()->selectCount($c);
            $us = $user->getSettings();
            if ($ac >= $us->getMaxSitesMaster()) {
                throw new WDPermissionException(sprintf(_("Sorry, you can be a master administrator of max %d Sites."), $us->getMaxSitesMaster()));
            }
        }
        return true;
    }

    public function getSitesAdminLeft($user)
    {
        $us = $user->getSettings();
        if (!$us->getMaxSitesAdmin() || $user->id == 1) {
            return null; // unlimited
        }
        $c = new Criteria();
        $c->add("user_id", $user->id);
        $c->addJoin("site_id", "site.site_id");
        $c->add('founder', false);
        $c->add("site.deleted", false);
        $ac = AdminPeer::instance()->selectCount($c);

        return max(array(0, $us->getMaxSitesAdmin() - $ac));
    }

    public function getSitesMasterLeft($user)
    {
        $us = $user->getSettings();
        if (!$us->getMaxSitesMaster() || $user->id == 1) {
            return null; // unlimited
        }
        $c = new Criteria();
        $c->add("user_id", $user->id);
        $c->addJoin("site_id", "site.site_id");
        $c->add('founder', true);
        $c->add("site.deleted", false);
        $ac = AdminPeer::instance()->selectCount($c);

        return max(array(0, $us->getMaxSitesMaster() - $ac));
    }

    /**
     * Checks if a site is public or user is a member of the site
     *
     * @param $user User
     * @param $site Site
     * @return boolean
     * @throws WDPermissionException
     */
    public function canAccessSite($user, $site)
    {
        // public or user is super
        if (! $site->getPrivate() || $user->id == 1) {
            return true;
        }
        // check if user is a member of the site
        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        $c->add("user_id", $user->id);

        if (MemberPeer::instance()->selectOne($c)) { // user is a member of the Wiki
            return true;
        }

        throw new WDPermissionException("User not allowed to access site");
    }

    private function permissionLookup($actionCode, $userCode, $permString)
    {
        $p = "/$actionCode:[a-z]*$userCode/";
        if (preg_match($p, $permString) == 0) {
            return false;
        } else {
            return true;
        }
    }

    private function handleFalse($message = null)
    {
        if (!$this->throwExceptions) {
            return false;
        } else {
            if ($message == null) {
                $message = "No permission";
            }
            throw new WDPermissionException($message);
        }
    }

    private function generateMessage($ac, $uc, $permString, $mode = 'page', $extraUserDesc = null)
    {
        if ($mode == 'page') {
            $actionString = self::$pageActionsDesc[$ac];
            $actionCode =  self::$pageActions[$ac];
        } elseif ($mode='forum') {
            $actionString = self::$forumActionsDesc[$ac];
            $actionCode =  self::$forumActions[$ac];
        }
        $allowedUsers = array();

        if (preg_match("/^.*?$actionCode:([a-z]*).*$/", $permString) !== 0) {
            $a2 = preg_replace("/^.*?$actionCode:([a-z]*).*$/", "\\1", $permString);
            for ($i = 0; $i<strlen($a2); $i++) {
                if ($extraUserDesc && $extraUserDesc[$a2[$i]]) {
                    $allowedUsers[] = $extraUserDesc[$a2[$i]];
                } else {
                    $allowedUsers[] = self::$userClassesDesc[array_search($a2[$i], self::$userClasses)];
                }
            }
        }
        $allowedUsers[] = _('administrators and moderators');
        //$actionString = array_searchself::ac
        $m = _('Sorry, you can not ').' '.$actionString.'. ' .
                _('Only ').' '.implode(', ', $allowedUsers).' '._(' are allowed to.');
        return $m;
    }

    public function setThrowExceptions($val)
    {
        $this->throwExceptions = $val;
    }

    private function shareSites($user1, $user2)
    {
        $db = Database::connection();
        $q = "SELECT member.* FROM member, member AS member2 WHERE member.user_id = '".$user1->id."' " .
                    "AND member2.user_id = '".$user2->id."' " .
                    "AND member.site_id = member2.site_id " .
                    "LIMIT 1";
        $r = $db->query($q);
        if ($r->nextRow()) {
            return true;
        } else {
            return false;
        }
    }
    /**
     * Check if user1 is an admin/mod of one of the sites user2 is a member of.
     */
    private function isUserSuperior($user1, $user2)
    {
        $db = Database::connection();
        $q = "SELECT member.* FROM member, admin WHERE member.user_id = '".$user2->id."' " .
                    "AND admin.user_id = '".$user1->id."' " .
                    "AND member.site_id = admin.site_id " .
                    "LIMIT 1";
        $r = $db->query($q);
        if ($r->nextRow()) {
            return true;
        }
        $q = "SELECT member.* FROM member, moderator WHERE member.user_id = '".$user2->id."' " .
                    "AND moderator.user_id = '".$user1->id."' " .
                    "AND member.site_id = moderator.site_id " .
                    "LIMIT 1";
        $r = $db->query($q);
        if ($r->nextRow()) {
            return true;
        }
        return false;
    }

    public function setCheckIpBlocks($val)
    {
        $this->checkIpBlocks = $val;
    }

    public function setCheckUserBlocks($val)
    {
        $this->checkUserBlocks = $val;
    }

    private function checkIpBlocked($ipString, $site)
    {
        $c = new Criteria();

        $ips = explode("|", $ipString);
        $q = "SELECT * FROM ip_block WHERE site_id='".$site->getSiteId()."' " .
                "AND (ip <<= '".db_escape_string($ips[0])."' ";
        if ($ips[1]) {
            $q.=  "OR ip <<= '".db_escape_string($ips[1])."'";
        }
        $q .= ")";
        $c->setExplicitQuery($q);
        $blocks = IpBlockPeer::instance()->select($c);
        return $blocks;
    }

    private function checkUserBlocked($user, $site)
    {
        $c = new Criteria();

        $c->add("site_id", $site->getSiteId());
        $c->add("user_id", $user->id);
        $block = UserBlockPeer::instance()->selectOne($c);
        return $block;
    }
}
