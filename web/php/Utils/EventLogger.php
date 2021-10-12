<?php

namespace Wikidot\Utils;

use Ozone\Framework\ODate;
use Ozone\Framework\Ozone;
use Wikidot\DB\LogEvent;

class EventLogger
{

    private static $instance;

    public static function instance()
    {
        if (self::$instance == null) {
            self::$instance = new EventLogger();
        }
        return self::$instance;
    }

    private function newEvent()
    {
        $event = new LogEvent();
        $event->setDate(new ODate());

        // now to make things easier dig into some global variables and set what is needed
        $runData = OZONE::getRunData();

        //site

        $site = $runData->getTemp("site");

        $event->setSiteId($site->getSiteId());
        $event->setTemp("site", $site);

        // user_id (if any)
        $event->setUserId($runData->getUserId());
        $event->setTemp("user", $runData->getUser());

        //ip address

        list($ip, $proxy) = explode("|", $runData->createIpString());
        $event->setIp($ip);
        $event->setProxy($proxy);

        // user agent

        $event->setUserAgent($_SERVER['HTTP_USER_AGENT']);

        return $event;
    }
}
