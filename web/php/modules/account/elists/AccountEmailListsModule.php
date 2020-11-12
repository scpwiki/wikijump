<?php
use DB\SitePeer;
use DB\EmailListPeer;

class AccountEmailListsModule extends AccountBaseModule
{

    public function build($runData)
    {

        $pl = $runData->getParameterList();
        $totalAll = (bool) $pl->getParameterValue('totalAll');

        $user = $runData->getUser();
        $c = new Criteria();
        if ($totalAll) {
            $q = "SELECT site.* FROM site, member WHERE member.user_id = '{$user->getUserId()}' AND member.site_id = site.site_id " .
                    "ORDER BY site.name";
            $c->setExplicitQuery($q);
            $ss = SitePeer::instance()->select($c);
            $sites = array();
            foreach ($ss as $s) {
                $sites[$s->getUnixName()] = array('site' => $s);
            }
        } else {
            $q = "SELECT email_list.* FROM email_list, email_list_subscriber, site WHERE email_list_subscriber.user_id = {$user->getUserId()} " .
                    "AND email_list_subscriber.list_id = email_list.list_id AND email_list.site_id = site.site_id " .
                    "ORDER BY site.name, email_list.title";
            $c->setExplicitQuery($q);

            $lists = EmailListPeer::instance()->select($c);

            // sorry  for the DIIIIRTY STYLE!!!
            $sites = array();
            foreach ($lists as $l) {
                $s = SitePeer::instance()->selectByPrimaryKey($l->getSiteId());
                if (!isset($sites[$s->getUnixName()])) {
                    $sites[$s->getUnixName()] = array('site' => $s, 'lists' => array());
                }
                $sites[$s->getUnixName()]['lists'][] = $l;
                $l->setTemp('site', $s);
            }
        }
        $runData->contextAdd('lists', $lists);
        $runData->contextAdd('sites', $sites);
        $runData->contextAdd('totalAll', $totalAll);
        $runData->contextAdd('user', $user);
    }
}
