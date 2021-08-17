<?php

namespace Wikidot\DB;


use Illuminate\Support\Facades\Cache;
use Wikidot\Utils\GlobalProperties;
use Ozone\Framework\Ozone;
use Wikidot\Utils\WDRenderUtils;
use Wikijump\Models\User;

/**
 * Object Model Class.
 *
 */
class Notification extends NotificationBase
{

    /**
     * Generates notification title based on the type
     */
    public function getTitle()
    {
        $type = $this->getType();
        switch ($type) {
            case 'new_private_message':
                $title = _("New private message");
                break;
            case 'new_membership_invitation':
                $title = _("New membership invitation");
                break;
            case 'removed_from_members':
                $title = _("Membership removal");
                break;
            case 'added_to_moderators':
                $title = _("Added to moderators");
                break;
            case 'removed_from_moderators':
                $title = _("Removed from moderators");
                break;
            case 'added_to_administrators':
                $title = _("Added to administrators");
                break;
            case 'removed_from_administrators':
                $title = _("Removed from administrators");
                break;
            case 'membership_application_accepted':
                $title = _("Membership application accepted");
                break;
            case 'membership_application_declined':
                $title = _("Membership application declined");
                break;
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
        $key = "notificationsfeed..".$this->getUserId();
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
            case 'new_private_message':
                $fromUser = User::find($extra['from_user_id']);
                $body = _('You have a new private message in your <a href="'.GlobalProperties::$HTTP_SCHEMA . "://" . GlobalProperties::$URL_HOST . '/account:you/start/messages">Inbox</a>.').'<br/>';
                $body .= _("From").": ".WDRenderUtils::renderUser($fromUser)."<br/>";
                $body .= _('Subject').': <a href="'.GlobalProperties::$HTTP_SCHEMA . "://" . GlobalProperties::$URL_HOST . '/account:you/start/messages/inboxmessage/'.$extra['message_id'].'">'.htmlspecialchars($extra['subject']).'</a><br/>';
                $body .= _('Preview (first few words)').': '.$extra['preview'];
                break;
            case 'new_membership_invitation':
                $body = _('You have received an invitation to join members of the site').' <a href="'.GlobalProperties::$HTTP_SCHEMA . "://" . $extra['site_domain'].'">"'.htmlspecialchars($extra['site_name']).'"</a>.';
                break;
            case 'removed_from_members':
                $body = _('You have been removed from members of the site').' <a href="'.GlobalProperties::$HTTP_SCHEMA . "://" . $extra['site_domain'].'">"'.htmlspecialchars($extra['site_name']).'"</a>.';
                break;
            case 'added_to_moderators':
                $body = _('You have been added to moderators of the site').' <a href="'.GlobalProperties::$HTTP_SCHEMA . "://" . $extra['site_domain'].'">"'.htmlspecialchars($extra['site_name']).'"</a>.';
                break;
            case 'removed_from_moderators':
                $body = _('You have been removed from moderators of the site').' <a href="'.GlobalProperties::$HTTP_SCHEMA . "://" . $extra['site_domain'].'">"'.htmlspecialchars($extra['site_name']).'"</a>.';
                break;
            case 'added_to_administrators':
                $body = _('You have been added to administrators of the site').' <a href="'.GlobalProperties::$HTTP_SCHEMA . "://" . $extra['site_domain'].'">"'.htmlspecialchars($extra['site_name']).'"</a>.';
                break;
            case 'removed_from_administrators':
                $body = _('You have been removed from administrators of the site').' <a href="'.GlobalProperties::$HTTP_SCHEMA . "://" . $extra['site_domain'].'">"'.htmlspecialchars($extra['site_name']).'"</a>.';
                break;
            case 'membership_application_accepted':
                $body = _('Your membership application to the site').' <a href="'.GlobalProperties::$HTTP_SCHEMA . "://" . $extra['site_domain'].'">"'.htmlspecialchars($extra['site_name']).'"</a>.'.
                    'has been accepted. You are now a member of this site.';
                break;
            case 'membership_application_declined':
                $body = _('Your membership application to the site').' <a href="'.GlobalProperties::$HTTP_SCHEMA . "://" . $extra['site_domain'].'">"'.htmlspecialchars($extra['site_name']).'"</a>.'.
                    'has been declined.';
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

        switch ($type) {
            case 'new_private_message':
                $urls = array(  array(_('read the message'),GlobalProperties::$HTTP_SCHEMA . "://" . GlobalProperties::$URL_HOST . '/account:you/start/messages/inboxmessage/'.$extra['message_id']),
                                array(_('inbox folder'), GlobalProperties::$HTTP_SCHEMA . "://" . GlobalProperties::$URL_HOST . '/account:you/start/messages'));
                break;
            case 'new_membership_invitation':
                $urls = array(array(_('view invitation'), GlobalProperties::$HTTP_SCHEMA . "://" . GlobalProperties::$URL_HOST . '/account:you/start/invitations'));
                break;
            case 'removed_from_members':
                $urls = array(array(_('sites you are a member of'), GlobalProperties::$HTTP_SCHEMA . "://" . GlobalProperties::$URL_HOST . '/account:you/start/memberof'));
                break;
            case 'added_to_moderators':
                $urls = array(array(_('sites you moderate'), GlobalProperties::$HTTP_SCHEMA . "://" . GlobalProperties::$URL_HOST . '/account:you/start/moderatorof'));
                break;
            case 'removed_from_moderators':
                $urls =  array(array(_('sites you moderate'), GlobalProperties::$HTTP_SCHEMA . "://" . GlobalProperties::$URL_HOST . '/account:you/start/moderatorof'));
                break;
            case 'added_to_administrators':
                $urls = array(array(_('sites you administer'), GlobalProperties::$HTTP_SCHEMA . "://" . GlobalProperties::$URL_HOST . '/account:you/start/adminof'));
                break;
            case 'removed_from_administrators':
                $urls = array(array(_('sites you administer'), GlobalProperties::$HTTP_SCHEMA . "://" . GlobalProperties::$URL_HOST . '/account:you/start/adminof'));
                break;
            case 'membership_application_accepted':
                $urls = array(  array(_('your applications'), GlobalProperties::$HTTP_SCHEMA . "://" . GlobalProperties::$URL_HOST . '/account:you/start/applications'),
                        array(_('sites you are a member of'), GlobalProperties::$HTTP_SCHEMA . "://" . GlobalProperties::$URL_HOST . '/account:you/start/memberof'));
                break;
            case 'membership_application_declined':
                $urls = array(  array(_('your applications'), GlobalProperties::$HTTP_SCHEMA . "://" . GlobalProperties::$URL_HOST . '/account:you/start/applications'),
                        array(_('sites you are a member of'), GlobalProperties::$HTTP_SCHEMA . "://" . GlobalProperties::$URL_HOST . '/account:you/start/memberof'));

                break;
        }
        return $urls;
    }

    public function getLocalizedExtra()
    {
        $extra =    unserialize(parent::getExtra());
        // ???
    }
}
