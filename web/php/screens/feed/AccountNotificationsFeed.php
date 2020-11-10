<?php
use DB\NotificationPeer;

class AccountNotificationsFeed extends FeedScreen
{

    protected $requiresAuthentication = true;

    public function render($runData)
    {
        $user = $runData->getTemp("user");
        $key = "notificationsfeed..".$user->getUserId();
        $mc = OZONE::$memcache;
        $out = $mc->get($key);
        if ($out) {
            return $out;
        }
        $out = parent::render($runData);
        $mc->set($key, $out, 0, 3600);
        return $out;
    }

    public function build($runData)
    {

        $user = $runData->getTemp("user");
        $userId = $user->getUserId();

        // set language for the user
        $lang = $user->getLanguage();
        $runData->setLanguage($lang);
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

        // now just get notifications for the user...

        $c = new Criteria();
        $c->add("user_id", $userId);
        $c->addOrderDescending('notification_id');
        $c->setLimit(20);

        $nots = NotificationPeer::instance()->select($c);

        $channel['title'] = sprintf(_('%s account notifications for user'), GlobalProperties::$SERVICE_NAME).' "'.htmlspecialchars($user->getNickName()).'"';
        $channel['link'] = GlobalProperties::$HTTP_SCHEMA . "://" . GlobalProperties::$URL_HOST . "/account:you/start/notifications";

        $items = array();

        foreach ($nots as $not) {
            $extra = $not->getExtra();
            $item = array();

            $item['title'] = $not->getTitle();
            switch ($not->getType()) {
                case "new_private_message":
                    $item['link'] = GlobalProperties::$HTTP_SCHEMA . "://" . GlobalProperties::$URL_HOST . "/account:you/start/messages/inboxmessage/".$extra['message_id'];
                    break;
                case "new_membership_invitation":
                    $item['link'] = GlobalProperties::$HTTP_SCHEMA . "://" . GlobalProperties::$URL_HOST . "/account:you/start/invitations";
                    break;
                case 'membership_application_accepted':
                    $item['link'] = GlobalProperties::$HTTP_SCHEMA . "://" . GlobalProperties::$URL_HOST . "/account:you/start/applications";
                    break;
                case 'membership_application_declined':
                    $item['link'] = GlobalProperties::$HTTP_SCHEMA . "://" . GlobalProperties::$URL_HOST . "/account:you/start/applications";
                    break;
                default:
                    $item['link'] = GlobalProperties::$HTTP_SCHEMA . "://" . GlobalProperties::$URL_HOST . "/account:you/start/notifications"."#notification-".$not->getNotificationId();
                    ;
            }

            $body = $not->getBody();

            $body = preg_replace('/onclick="[^"]+"/', '', $body);

            $item['description'] = $body;

            $item['guid'] = $channel['link']."#notification-".$not->getNotificationId();
            $item['date'] = date('r', $not->getDate()->getTimestamp());
            // TODO: replace relative links with absolute links!
            $content =  '';

            $items[] = $item;
        }

        $runData->contextAdd("channel", $channel);
        $runData->contextAdd("items", $items);
    }
}
