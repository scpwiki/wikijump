<?php
use DB\NotificationPeer;

class AccountNotificationsListModule extends AccountBaseModule
{

    public function build($runData)
    {

        $userId = $runData->getUserId();
        $pl = $runData->getParameterList();
        $pageNumber = $pl->getParameterValue("page");

        if ($pageNumber == null || !is_numeric($pageNumber) || $pageNumber <1) {
            $pageNumber = 1;
        }

        // now just get notifications for the user...

        $perPage = 30;

        $offset = ($pageNumber - 1)*$perPage;
        $count = $perPage*2 + 1;

        $c = new Criteria();
        $c->add("user_id", $userId);
        $c->addOrderDescending('notification_id');
        $c->setLimit($count, $offset);

        $nots = NotificationPeer::instance()->select($c);

        // now see if number of selected is equal $perPage + 1. If so -
        // there is at least 1 more page to show...
        $counted = count($nots);
        $pagerData = array();
        $pagerData['current_page'] = $pageNumber;
        if ($counted >$perPage*2) {
            $knownPages=$pageNumber + 2;
            $pagerData['known_pages'] = $knownPages;
        } elseif ($counted >$perPage) {
            $knownPages=$pageNumber + 1;
            $pagerData['total_pages'] = $knownPages;
        } else {
            $totalPages = $pageNumber;
            $pagerData['total_pages'] = $totalPages;
        }

        $nots = array_slice($nots, 0, $perPage);

        $runData->contextAdd("pagerData", $pagerData);

        $runData->contextAdd("notifications", $nots);
    }
}
