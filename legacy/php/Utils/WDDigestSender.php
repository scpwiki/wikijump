<?php

namespace Wikidot\Utils;

use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Database\Database;
use Ozone\Framework\Ozone;
use Ozone\Framework\OzoneEmail;
use Wikidot\DB\NotificationPeer;
use Wikijump\Models\UserMessage;

class WDDigestSender
{

    public function handleUser($user)
    {

        $db = Database::connection();
        $db->begin();

        $c = new Criteria();
        $c->add("user_id", $user->id);
        $c->add("notify_email", true);
        $c->addOrderAscending("notification_id");

        $nots = NotificationPeer::instance()->select($c);

        if (count($nots) == 0) {
            $db->commit();
            return;
        }

        if (count($nots)>0) {
            $q = "UPDATE notification SET notify_email=FALSE " .
                    "WHERE user_id='".$user->id."' AND " .
                    "notify_email = TRUE";
            $db->query($q);
        }

        // set language

        $lang = $user->language;
        OZONE::getRunData()->setLanguage($lang);

        $GLOBALS['lang'] = $lang;

        // and for gettext too:
        switch ($lang) {
            case 'pl':
                $glang="pl_PL";
                break;
            case 'en':
                $glang="en_US";
                break;
        }

        putenv("LANG=$glang");
        putenv("LANGUAGE=$glang");
        setlocale(LC_ALL, $glang.'.UTF-8');

        $nots2 = array();

        foreach ($nots as &$not) {
            if ($not->getType() == "new_private_message") {
                // check if the message is read or still new
                $extra = $not->getExtra();
                $pm = UserMessage::find($extra['message_id']);
                if ($pm && $pm->isUnread()) {
                    $body = $not->getBody();
                    $body = preg_replace('/<br\/>Preview.*$/sm', '', $body);
                    $body = preg_replace('/You have.*?<br\/>/sm', '', $body);
                    $not->setBody($body);
                    $nots2[] = $not;
                }
            } else {
                $nots2[] = $not;
            }
        }

        $count = count($nots2);

        // now send an email

        $oe = new OzoneEmail();
        $oe->addAddress($user->email);
        $oe->setSubject(sprintf(_("%s Account Notifications"), GlobalProperties::$SERVICE_NAME));
        $oe->contextAdd('user', $user);
        $oe->contextAdd('notifications', $nots2);
        $oe->contextAdd('count', $count);

        $oe->setBodyTemplate('DigestEmail');

        if (!$oe->send()) {
            throw new ProcessException("The email cannot be sent to address ".$user->email, "email_failed");
        }

        $db->commit();
    }
}
