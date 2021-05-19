<?php

namespace Wikidot\Actions;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Database\Database;
use Ozone\Framework\ODate;
use Ozone\Framework\RunData;
use Ozone\Framework\SmartyAction;
use Wikidot\Config\ForbiddenNames;
use Wikidot\DB\Admin;
use Wikidot\DB\MemberInvitationPeer;
use Wikidot\DB\Site;
use Wikidot\DB\SitePeer;
use Wikidot\DB\MemberPeer;
use Wikidot\DB\Member;
use Wikidot\DB\MembershipLink;
use Wikidot\DB\MemberApplicationPeer;
use Wikidot\DB\AdminPeer;
use Wikidot\DB\ModeratorPeer;
use Wikidot\Utils\AdminNotificationMaker;
use Wikidot\Utils\GlobalProperties;
use Wikidot\Utils\ProcessException;
use Wikidot\Utils\WDPermissionException;
use Wikidot\Utils\WDStringUtils;

class AccountMembershipAction extends SmartyAction
{

    public function isAllowed($runData)
    {
        $userid = $runData->getUserId();
        if(!$userid) {
            throw new WDPermissionException(_("Not allowed. You should login first."));
        }
        return true;
    }

    public function perform($runData)
    {
    }

    public function acceptInvitationEvent(RunData $runData)
    {
        $pl = $runData->getParameterList();
        $invitationId = $pl->getParameterValue("invitation_id");
        $user = $runData->getUser();

        $invitation = MemberInvitationPeer::instance()->selectByPrimaryKey($invitationId);
        $site = SitePeer::instance()->selectByPrimaryKey($invitation->getSiteId());
        if ($invitation == null || $invitation->getUserId() != $user->id || $site == null) {
            throw new ProcessException(_("Invitation cannot be found."), "no_invitation");
        }

        if ($site->getPrivate()) {
            $settings = $site->getSettings();
            $maxMembers = $settings->getMaxPrivateMembers();
            $c = new Criteria();
            $c->add("site_id", $site->getSiteId());
            $cmem = MemberPeer::instance()->selectCount($c);
            if ($cmem >= $maxMembers) {
                throw new ProcessException(sprintf(_('Sorry, at the moment max %d member limit apply for private Wikis. The Site would have to be upgraded to allow more members.'), $maxMembers));
            }
        }
        // all ok...

        $db = Database::connection();
        $db->begin();
        // create membership
        $member = new Member();
        $member->setUserId($user->id);
        $member->setSiteId($invitation->getSiteId());
        $member->setDateJoined(new ODate());

        $member->save();

        $ml = new MembershipLink();
        $ml->setUserId($user->id);
        $ml->setSiteId($invitation->getSiteId());
        $ml->setDate(new ODate());
        $ml->setType('INTERNAL_INVITATION');
        $ml->setByUserId($invitation->getByUserId());
        $ml->save();

        // remove application (if any)
        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        $c->add("user_id", $user->id);

        MemberApplicationPeer::instance()->delete($c);

        MemberInvitationPeer::instance()->deleteByPrimaryKey($invitationId);
        $runData->ajaxResponseAdd("message", _('Now you are a member of the site').' <a href='.GlobalProperties::$HTTP_SCHEMA . '://' . htmlspecialchars($site->getDomain()).'">'.htmlspecialchars($site->getName()).'</a>');

        AdminNotificationMaker::instance()->memberInvitationAccepted($site, $user);
        $db->commit();
    }

    public function throwAwayInvitationEvent($runData)
    {
        $pl = $runData->getParameterList();
        $invitationId = $pl->getParameterValue("invitation_id");
        $user = $runData->getUser();

        $db = Database::connection();
        $db->begin();

        $invitation = MemberInvitationPeer::instance()->selectByPrimaryKey($invitationId);
        $site = SitePeer::instance()->selectByPrimaryKey($invitation->getSiteId());
        if ($invitation == null || $invitation->getUserId() != $user->id || $site == null) {
            throw new ProcessException(_("Invitation cannot be found."), "no_invitation");
        }

        $c = new Criteria();
        $c->add("invitation_id", $invitationId);
        $c->add("user_id", $user->id);
        MemberInvitationPeer::instance()->delete($c);
        AdminNotificationMaker::instance()->memberInvitationDeclined($site, $user);
        $db->commit();
    }

    public function signOffEvent($runData)
    {
        // remove the membership AND adminship AND moderatorship

        $siteId = $runData->getParameterList()->getParameterValue("site_id");
        $user = $runData->getUser();
        $c = new Criteria();
        $c->add("user_id", $user->id);
        $c->add("site_id", $siteId);

        $db = Database::connection();
        $db->begin();

        // check if admin

        /**
         * @var Admin|null $admin
         */
        $admin =  AdminPeer::instance()->selectOne($c) ?? null;

        if ($admin && $admin->getFounder()) {
            throw new ProcessException(_("You have founded this site - sorry, you cannot resign."), "founder_nonremovable");
        }

        if ($admin) {
            // check if not the last admin!!!
            $c2 = new Criteria();
            $c2->add("site_id", $siteId);
            $acount = AdminPeer::instance()->selectCount($c2);
            if ($acount == 1) {
                $runData->ajaxResponseAdd("status", "last_admin");
                $runData->ajaxResponseAdd("message", _("You cannot simply resign - you are the last admin of this site!"));
                $db->commit();
                return;
            }
        }

        MemberPeer::instance()->delete($c);
        ModeratorPeer::instance()->delete($c);
        AdminPeer::instance()->delete($c);

        $site = SitePeer::instance()->selectByPrimaryKey($siteId);
        AdminNotificationMaker::instance()->memberResigned($site, $user);

        $db->commit();
    }

    public function adminResignEvent($runData)
    {
        $siteId = $runData->getParameterList()->getParameterValue("site_id");
        $user = $runData->getUser();

        $db = Database::connection();
        $db->begin();

        $c = new Criteria();
        $c->add("user_id", $user->id);
        $c->add("site_id", $siteId);
        $admin = AdminPeer::instance()->selectOne($c);

        if ($admin && $admin->getFounder()) {
            throw new ProcessException(_("You have founded this site - sorry, you cannot resign."), "founder_nonremovable");
        }

        // you cannot resign if you are the last admin...
        $c2 = new Criteria();
        $c2->add("site_id", $siteId);
        $acount = AdminPeer::instance()->selectCount($c2);
        if ($acount == 1) {
            $runData->ajaxResponseAdd("status", "last_admin");
            $runData->ajaxResponseAdd("message", _("You cannot simply resign - you are the last admin of this site!"));
            $db->commit();
            return;
        }

        $c = new Criteria();
        $c->add("user_id", $user->id);
        $c->add("site_id", $siteId);
        AdminPeer::instance()->delete($c);

        $site = SitePeer::instance()->selectByPrimaryKey($siteId);
        $user = $runData->getUser();
        AdminNotificationMaker::instance()->adminResigned($site, $user);

        $db->commit();
    }

    public function moderatorResignEvent($runData)
    {
        $siteId = $runData->getParameterList()->getParameterValue("site_id");
        $user = $runData->getUser();

        $db = Database::connection();
        $db->begin();

        $c = new Criteria();
        $c->add("user_id", $user->id);
        $c->add("site_id", $siteId);
        ModeratorPeer::instance()->delete($c);

        $site = SitePeer::instance()->selectByPrimaryKey($siteId);
        $user = $runData->getUser();
        AdminNotificationMaker::instance()->moderatorResigned($site, $user);

        $db->commit();
    }

    public function removeApplicationEvent($runData)
    {
        $siteId = $runData->getParameterList()->getParameterValue("site_id");
        $user = $runData->getUser();

        $db = Database::connection();
        $db->begin();

        $c = new Criteria();
        $c->add("user_id", $user->id);
        $c->add("site_id", $siteId);
        MemberApplicationPeer::instance()->delete($c);

        $db->commit();
    }

    public function restoreSiteEvent($runData)
    {
        $pl = $runData->getParameterList();
        $siteId = $pl->getParameterValue('siteId');
        $unixName = trim($pl->getParameterValue('unixName'));

        $c = new Criteria();
        $c->add('site_id', $siteId);
        $c->add('deleted', true);

        /**
         * @var Site $site
         */
        $site = SitePeer::instance()->selectOne($c);

        if (!$site) {
            throw new ProcessException(_('Error selecting a site to restore.'));
        }

        // check if allowed
        $user = $runData->getUser();

        $c = new Criteria();
        $c->add("user_id", $user->id);
        $c->add("site_id", $site->getSiteId());
        $c->add("founder", true);
        $rel = AdminPeer::instance()->selectOne($c);

        if (!$rel) {
            throw new ProcessException(_("Sorry, you have no permissions to restore this site."));
        }

        $db = Database::connection();
        $db->begin();

        // validate unix name
        $errors = array();
        if ($unixName === null || strlen($unixName)<3 || strlen(WDStringUtils::toUnixName($unixName)) < 3) {
            $errors['unixname'] = _("Web address must be present and should be at least 3 characters long.");
        } elseif (strlen($unixName)>30) {
            $errors['unixname']  = _("Web address name should not be longer than 30 characters.");
        } elseif (preg_match("/^[a-z0-9\-]+$/", $unixName) == 0) {
            $errors['unixname'] = _('Only lowercase alphanumeric and "-" (dash) characters allowed in the web address.');
        } elseif (preg_match("/\-\-/", $unixName) !== 0) {
            $errors['unixname'] = _('Only lowercase alphanumeric and "-" (dash) characters allowed in the web address. Double-dash (--) is not allowed.');
        } else {
            $unixName = WDStringUtils::toUnixName($unixName);

            if (!$runData->getUser()->id == 1) {
                //  handle forbidden names
                foreach (ForbiddenNames::$sites as $regex) {
                    if (preg_match($regex, $unixName) > 0) {
                        $errors['unixname'] = _('This web address is not allowed or reserved.');
                    }
                }
            }

            // check if the domain is not taken.
            $c = new Criteria();
            $c->add("unix_name", $unixName);
            $ss = SitePeer::instance()->selectOne($c) ?? null;
            if ($ss) {
                $errors['unixname'] = _('Sorry, this web address is already used by another site.');
            }
        }

        if (isset($errors['unixname'])) {
            throw new ProcessException($errors['unixname']);
        }

        $oldUnixName = $site->getUnixName();
        $oldLocalPath = $site->getLocalFilesPath();
        $site->setUnixName($unixName);
        //  rename the files
        mkdirfull(dirname($site->getLocalFilesPath()));
        @rename($oldLocalPath, $site->getLocalFilesPath());

        $site->setDeleted(false);
        $site->setCustomDomain(null);
        $site->save();

        $db->commit();

        $runData->ajaxResponseAdd('unixName', $site->getUnixName());
    }
}
