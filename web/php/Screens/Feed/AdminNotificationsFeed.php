<?php

namespace Wikidot\Screens\Feed;

use Illuminate\Support\Facades\Cache;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Ozone;
use Wikidot\DB\AdminPeer;
use Wikidot\DB\AdminNotificationPeer;
use Wikidot\Utils\FeedScreen;
use Wikidot\Utils\GlobalProperties;

class AdminNotificationsFeed extends FeedScreen
{

    protected $requiresAuthentication = true;

    public function render($runData)
    {
        $user = $runData->getTemp("user");
        $site = $runData->getTemp("site");

        // check if site admin

        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        $c->add("user_id", $user->id);

        $admin = AdminPeer::instance()->selectOne($c);

        if ($admin == null) {
            return _("Sorry, you are not allowed to view this feed.");
        }

        $key = "adminnotificationsfeed..".$site->getSiteId();
        $out = Cache::get($key);
        if ($out) {
            return $out;
        }
        $out = parent::render($runData);
        Cache::put($key, $out, 3600);
        return $out;
    }

    public function build($runData)
    {

        $site = $runData->getTemp("site");

        // now just get notifications for the site...

        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        $c->addOrderDescending('notification_id');
        $c->setLimit(20);

        $nots = AdminNotificationPeer::instance()->select($c);

        $channel['title'] = _('Admin notifications for site').' "'.htmlspecialchars($site->getName()).'"';
        $channel['link'] = GlobalProperties::$HTTP_SCHEMA . "://" . $site->getDomain()."/admin:manage/start/notifications";

        $items = array();

        foreach ($nots as $not) {
            $extra = $not->getExtra();
            $item = array();

            $item['title'] = $not->getTitle();
            switch ($not->getType()) {
                case "NEW_MEMBER_APPLICATION":
                    $item['link'] = GlobalProperties::$HTTP_SCHEMA . "://" . $site->getDomain()."/admin:manage/start/ma";
                    break;

                default:
                    $item['link'] = GlobalProperties::$HTTP_SCHEMA . "://" . $site->getDomain()."/admin:manage/start/notifications"."#notification-".$not->getNotificationId();
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
