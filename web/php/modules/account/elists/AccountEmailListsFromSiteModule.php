<?php
use DB\SitePeer;
use DB\EmailListPeer;
use DB\EmailListSubscriberPeer;

class AccountEmailListsFromSiteModule extends AccountBaseModule
{

    public function build($runData)
    {
        $user = $runData->getUser();
        $c = new Criteria();

        $pl = $runData->getParameterList();
        $siteId = $pl->getParameterValue('siteId');

        $all = (bool) $pl->getParameterValue('all');

        $site = SitePeer::instance()->selectByPrimaryKey($siteId);
        if ($all) {
            $q = "SELECT email_list.* FROM email_list WHERE " .
                    "email_list.site_id = '{$site->getSiteId()}' " .
                    "ORDER BY email_list.title";
            $c->setExplicitQuery($q);

            $lists = EmailListPeer::instance()->select($c);
            // check if subscribed
            foreach ($lists as $list) {
                $c2 = new Criteria();
                $c2->add('user_id', $user->getUserId());
                $c2->add('list_id', $list->getListId());
                $sub = EmailListSubscriberPeer::instance()->selectOne($c2);
                if ($sub) {
                    $list->setTemp('subscribed', true);
                }
            }
        } else {
            // only subscribed
            $q = "SELECT email_list.* FROM email_list, email_list_subscriber WHERE email_list_subscriber.user_id = {$user->getUserId()} " .
                "AND email_list_subscriber.list_id = email_list.list_id AND email_list.site_id = '{$site->getSiteId()}' " .
                "ORDER BY email_list.title";
            $c->setExplicitQuery($q);

            $lists = EmailListPeer::instance()->select($c);
            foreach ($lists as $list) {
                $list->setTemp('subscribed', true);
            }
        }

        $runData->contextAdd('all', $all);
        $runData->contextAdd('lists', $lists);
        $runData->contextAdd('site', $site);
    }
}
