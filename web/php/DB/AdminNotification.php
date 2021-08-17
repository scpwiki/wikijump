<?php

namespace Wikidot\DB;


use Illuminate\Support\Facades\Cache;
use Ozone\Framework\Ozone;
use Wikidot\Utils\WDRenderUtils;
use Wikidot\Utils\GlobalProperties;
use Wikijump\Models\User;

/**
 * Object Model mapped Class.
 *
 */
class AdminNotification extends AdminNotificationBase
{

        /**
     * Generates notification title based on the type
     */
    public function getTitle()
    {
        $type = $this->getType();
        switch ($type) {
            case 'NEW_MEMBER_APPLICATION':
                $title = _("New member application");
                break;
            case 'INVITATION_ACCEPTED':
                $title = _("Membership invitation accepted");
                break;
            case 'INVITATION_DECLINED':
                $title = _("Membership invitation declined");
                break;
            case 'NEW_MEMBER_BY_PASSWORD':
                $title = _("New member joined");
                break;
            case 'MEMBER_RESIGNED':
                $title = _("A member has left");
                break;
            case 'MODERATOR_RESIGNED':
                $title = _("A moderator resigned");
                break;
            case 'ADMIN_RESIGNED':
                $title = _("An administrator resigned");
                break;
            case 'NEW_MEMBER_BY_EMAIL_INVITATION':
                $title = _("Email invitation accepted");
        }

        return $title;
    }

    public function setExtra($data)
    {
        parent::setExtra(serialize($data));
    }

    public function getExtra()
    {
        return unserialize(pg_unescape_bytea(parent::getExtra()));
    }

    public function save()
    {
        $key = "adminnotificationsfeed..".$this->getSiteId();
        Cache::forget($key);
        return parent::save();
    }

    public function getBody()
    {

        if (parent::getBody() != "") {
            return parent::getBody();
        }
        $type = $this->getType();
        $extra = $this->getExtra();
        $lang = OZONE::getRunData()->getLanguage();
        switch ($type) {
            case 'NEW_MEMBER_APPLICATION':
                $user = User::find($extra['from_user_id']);
                $body = sprintf(_('There is a new member application from user %s.'), WDRenderUtils::renderUser($user));
                break;
            case 'INVITATION_ACCEPTED':
                $user = User::find($extra['user_id']);
                $body = sprintf(_('The user %s has accepted the invitation and is now a member of the site.'), WDRenderUtils::renderUser($user));
                break;
            case 'INVITATION_DECLINED':
                $user = User::find($extra['user_id']);
                $body = sprintf(_('The user %s has not accepted the invitation.'), WDRenderUtils::renderUser($user));
                break;
            case 'NEW_MEMBER_BY_PASSWORD':
                $user = User::find($extra['user_id']);
                $body = sprintf(_('A new member joined the site: %s - by providing a valid membership password.'), WDRenderUtils::renderUser($user));
                break;
            case 'NEW_MEMBER_BY_EMAIL_INVITATION':
                $user = User::find($extra['user_id']);
                $body = sprintf(_('A new user (%s) accepted the invitation and is now a member of the Site.'), WDRenderUtils::renderUser($user));
                break;
            case 'MEMBER_RESIGNED':
                $user = User::find($extra['user_id']);
                $body = sprintf(_('The user %s is no longer a site member. Resigned.'), WDRenderUtils::renderUser($user));
                break;
            case 'MODERATOR_RESIGNED':
                $user = User::find($extra['user_id']);
                $body = sprintf(_('The user %s resigned from being a moderator of this site.'), WDRenderUtils::renderUser($user));
                break;
            case 'ADMIN_RESIGNED':
                $user = User::find($extra['user_id']);
                $body = sprintf(_('The user %s resigned from being an administrator of this site.'), WDRenderUtils::renderUser($user));
                break;
        }

        return $body;
    }

    public function getUrls()
    {
        $type = $this->getType();
        $extra = $this->getExtra();
        if ($extra['urls']) {
            return  $extra['urls'];
        }
        $lang = OZONE::getRunData()->getLanguage();
        $site = OZONE::getRunData()->getTemp("site");

        switch ($type) {
            case 'NEW_MEMBER_APPLICATION':
                $urls =  array( array(_('check pending applications'),GlobalProperties::$HTTP_SCHEMA . "://" . $site->getDomain().'/admin:manage/start/ma')
                                );
                break;
            case 'INVITATION_ACCEPTED':
                $urls  = array( array(_('site members'),GlobalProperties::$HTTP_SCHEMA . "://" . $site->getDomain().'/admin:manage/start/members-list')
                                );
                break;
            case 'INVITATION_DECLINED':
                break;
            case 'NEW_MEMBER_BY_PASSWORD':
                $urls = array(  array('_(site members)',GlobalProperties::$HTTP_SCHEMA . "://" . $site->getDomain().'/admin:manage/start/members-list')
                            );
                break;
            case 'MEMBER_RESIGNED':
                $urls = array(  array(_('site members'),GlobalProperties::$HTTP_SCHEMA . "://" . $site->getDomain().'/admin:manage/start/members-list')
                                );
                break;
            case 'MODERATOR_RESIGNED':
                $urls = array(  array(_('site moderators'),GlobalProperties::$HTTP_SCHEMA . "://" . $site->getDomain().'/admin:manage/start/moderators'),
                    array(_('site members'),GlobalProperties::$HTTP_SCHEMA . "://" . $site->getDomain().'/admin:manage/start/members-list')
                                );
                break;
            case 'ADMIN_RESIGNED':
                $urls = array(  array(_('site adminitrators'),GlobalProperties::$HTTP_SCHEMA . "://" . $site->getDomain().'/admin:manage/start/admins'),
                    array(_('site members'),GlobalProperties::$HTTP_SCHEMA . "://" . $site->getDomain().'/admin:manage/start/members-list')
                                );
                break;
        }
        return $urls;
    }
}
