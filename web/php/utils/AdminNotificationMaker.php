<?php
use DB\SitePeer;
use DB\AdminNotification;

class AdminNotificationMaker
{

    private static $instance;

    public static function instance()
    {
        if (self::$instance == null) {
            self::$instance = new AdminNotificationMaker();
        }
        return self::$instance;
    }

    public function newMemberApplication($application)
    {
        $user = $application->getUser();

        $siteId = $application->getSiteId();
        $site = SitePeer::instance()->selectByPrimaryKey($siteId);

        $not = new AdminNotification();
        $not->setSiteId($site->getSiteId());

        $not->setType("NEW_MEMBER_APPLICATION");

        $not->setDate(new ODate());

        $extra = array();
        $extra['application_id'] = $application->getApplicationId();
        $extra['from_user_id'] = $user->getUserId();
        //$extra['urls'] = array(   array('check pending applications','http://'.$site->getDomain().'/admin:manage/start/ma')

        /*
         * format for urls is:
         * 0 - anchor
         * 1 - href
         * 2 - onclick
         */

        $not->setExtra($extra);

        $not->save();
    }

    public function newMemberByPassword($site, $user)
    {

        $not = new AdminNotification();
        $not->setSiteId($site->getSiteId());

        $not->setType("NEW_MEMBER_BY_PASSWORD");

        $not->setDate(new ODate());

        $extra = array();
        $extra['user_id'] = $user->getUserId();
        //$extra['urls'] = array(   array('site members','http://'.$site->getDomain().'/admin:manage/start/members-list')
        $not->setExtra($extra);

        $not->save();
    }

    public function acceptedEmailInvitation($inv, $user)
    {

        $site = SitePeer::instance()->selectByPrimaryKey($inv->getSiteId());

        $not = new AdminNotification();
        $not->setSiteId($site->getSiteId());

        $not->setType("NEW_MEMBER_BY_EMAIL_INVITATION");

        $not->setDate(new ODate());

        $extra = array();
        $extra['user_id'] = $user->getUserId();
        //$extra['urls'] = array(   array('site members','http://'.$site->getDomain().'/admin:manage/start/members-list')
        $not->setExtra($extra);

        $not->save();
    }

    public function memberResigned($site, $user)
    {

        $not = new AdminNotification();
        $not->setSiteId($site->getSiteId());

        $not->setType("MEMBER_RESIGNED");

        $not->setDate(new ODate());

        $extra = array();
        $extra['user_id'] = $user->getUserId();
        //$extra['urls'] = array(   array('site members','http://'.$site->getDomain().'/admin:manage/start/members-list')
        $not->setExtra($extra);

        $not->save();
    }

    public function moderatorResigned($site, $user)
    {

        $not = new AdminNotification();
        $not->setSiteId($site->getSiteId());

        $not->setType("MODERATOR_RESIGNED");

        $not->setDate(new ODate());

        $extra = array();
        $extra['user_id'] = $user->getUserId();
        //$extra['urls'] = array(   array('site moderators','http://'.$site->getDomain().'/admin:manage/start/moderators'),
        //  array('site members','http://'.$site->getDomain().'/admin:manage/start/members-list')
        $not->setExtra($extra);

        $not->save();
    }

    public function adminResigned($site, $user)
    {

        $not = new AdminNotification();
        $not->setSiteId($site->getSiteId());

        $not->setType("ADMIN_RESIGNED");

        $not->setDate(new ODate());

        $extra = array();
        //$extra['urls'] = array(   array('site adminitrators','http://'.$site->getDomain().'/admin:manage/start/admins'),
        //  array('site members','http://'.$site->getDomain().'/admin:manage/start/members-list')
        $extra['user_id'] = $user->getUserId();
        $not->setExtra($extra);

        $not->save();
    }

    public function memberInvitationAccepted($site, $user)
    {
        $not = new AdminNotification();
        $not->setSiteId($site->getSiteId());

        $not->setType("INVITATION_ACCEPTED");

        $not->setDate(new ODate());

        $extra = array();
        //$extra['urls'] = array(   array('site members','http://'.$site->getDomain().'/admin:manage/start/members-list')
        $extra['user_id'] = $user->getUserId();
        $not->setExtra($extra);

        $not->save();
    }
    public function memberInvitationDeclined($site, $user)
    {
        $not = new AdminNotification();
        $not->setSiteId($site->getSiteId());

        $not->setType("INVITATION_DECLINED");

        $not->setDate(new ODate());

        $extra = array();
        $extra['user_id'] = $user->getUserId();
        //$extra['urls'] = array(   array('site members','http://'.$site->getDomain().'/admin:manage/start/members-list')
        $not->setExtra($extra);

        $not->save();
    }
}
