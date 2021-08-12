<?php

namespace Wikidot\Utils;

use Ozone\Framework\ODate;
use Wikidot\DB\Notification;
use Wikidot\DB\SitePeer;
use Wikijump\Models\UserMessage;

class NotificationMaker
{

    private static $instance;

    public static function instance()
    {
        if (self::$instance == null) {
            self::$instance = new NotificationMaker();
        }
        return self::$instance;
    }

    /**
     * Create a notification on a private message being sent.
     * @param UserMessage $message
     */
    public function privateMessageNotification(UserMessage $message)
    {
        $fromUser = $message->sender;

        $not = new Notification();
        $not->setUserId($message->recipient->id);
        $not->setType('new_private_message');

        $body = 'You have a new private message in your <a href="'.GlobalProperties::$HTTP_SCHEMA . '://' . GlobalProperties::$URL_HOST . '/account:you/start/messages">Inbox</a>!<br/>';
        $body .= 'From: ' .WDRenderUtils::renderUser($fromUser). '<br/>';
        $body .= 'Subject: <a href="'.GlobalProperties::$HTTP_SCHEMA . '://' . GlobalProperties::$URL_HOST . '/account:you/start/messages/inboxmessage/'.$message->id.'">'.htmlspecialchars($message->subject).'</a><br/>';
        $body .= 'Preview (first few words): ';
        $body .= $message->preview();

        $not->setBody($body);

        $not->setDate(new ODate());

        $extra = array();
        $extra['message_id'] = $message->id;
        $extra['from_user_id'] = $message->sender->id;
        $extra['subject'] = $message->subject;
        $extra['preview'] = $message->preview();

        $not->setExtra($extra);

        $not->save();
    }

    public function newMembershipInvitation($invitation)
    {
        $site = SitePeer::instance()->selectByPrimaryKey($invitation->getSiteId());
        $not = new Notification();
        $not->setUserId($invitation->getUserId());
        $not->setType('new_membership_invitation');

        $extra = array();
        $extra['site_id'] = $site->getSiteId();
        $extra['site_name'] = $site->getName();
        $extra['site_domain'] = $site->getDomain();
        $not->setExtra($extra);
        $not->setDate(new ODate());
        $not->save();
    }

    public function removedFromMembers($site, $user)
    {
        // and create a notification too...
        $not = new Notification();
        $not->setUserId($user->id);
        $not->setType('removed_from_members');

        $extra = array();
        $extra['site_id'] = $site->getSiteId();
        $extra['site_name'] = $site->getName();
        $extra['site_domain'] = $site->getDomain();

        $not->setExtra($extra);
        $not->setDate(new ODate());
        $not->save();
    }

    public function addedToModerators($site, $user)
    {
        // and create a notification too...
        $not = new Notification();
        $not->setUserId($user->id);
        $not->setType('added_to_moderators');

        $extra['site_id'] = $site->getSiteId();
        $extra['site_name'] = $site->getName();
        $extra['site_domain'] = $site->getDomain();

        $not->setExtra($extra);

        $not->setDate(new ODate());
        $not->save();
    }

    public function removedFromModerators($site, $user)
    {
        // and create a notification too...
        $not = new Notification();
        $not->setUserId($user->id);
        $not->setType('removed_from_moderators');

        $extra['site_id'] = $site->getSiteId();
        $extra['site_name'] = $site->getName();
        $extra['site_domain'] = $site->getDomain();
        $not->setExtra($extra);

        $not->setDate(new ODate());
        $not->save();
    }

    public function addedToAdmins($site, $user)
    {
        // and create a notification too...
        $not = new Notification();
        $not->setUserId($user->id);
        $not->setType('added_to_administrators');

        $extra['site_id'] = $site->getSiteId();
        $extra['site_name'] = $site->getName();
        $extra['site_domain'] = $site->getDomain();
        $not->setExtra($extra);

        $not->setDate(new ODate());
        $not->save();
    }

    public function removedFromAdmins($site, $user)
    {
        // and create a notification too...
        $not = new Notification();
        $not->setUserId($user->id);
        $not->setType('removed_from_administrators');

        $extra['site_id'] = $site->getSiteId();
        $extra['site_name'] = $site->getName();
        $extra['site_domain'] = $site->getDomain();
        $not->setExtra($extra);

        $not->setDate(new ODate());
        $not->save();
    }

    public function membershipApplicationAccepted($site, $user)
    {
        $not = new Notification();
        $not->setUserId($user->id);
        $not->setType('membership_application_accepted');

        $extra['site_id'] = $site->getSiteId();
        $extra['site_name'] = $site->getName();
        $extra['site_domain'] = $site->getDomain();
        $not->setExtra($extra);

        $not->setDate(new ODate());
        $not->save();
    }
    public function membershipApplicationDeclined($site, $user)
    {
        $not = new Notification();
        $not->setUserId($user->id);
        $not->setType('membership_application_declined');

        //$urls = array(    array('your applications', "https://www.wikijump.com/account:you/start/applications"),
        $extra['site_id'] = $site->getSiteId();
        $extra['site_name'] = $site->getName();
        $extra['site_domain'] = $site->getDomain();
        $not->setExtra($extra);

        $not->setDate(new ODate());
        $not->save();
    }
}
