<?php

namespace Wikidot\Modules\ManageSite\Elists;

use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Database\Database;
use Wikidot\DB\EmailListPeer;
use Wikidot\DB\OzoneUserPeer;
use Wikidot\Utils\ManageSiteBaseModule;
use Wikidot\Utils\ProcessException;

class ManageSiteEmailListSubscribersModule extends ManageSiteBaseModule
{

    public function build($runData)
    {
        $site = $runData->getTemp('site');
        $pl = $runData->getParameterList();
        $listId = $pl->getParameterValue("listId");

        $db = Database::connection();
        $db->begin();

        // get the list
        $c= new Criteria();
        $c->add('site_id', $site->getSiteId());
        $c->add('list_id', $listId);

        $list = EmailListPeer::instance()->selectOne($c);

        if (!$list) {
            throw new ProcessException('The requested list  cannot be found.');
        }

        // get all subscribers
        $q = "SELECT ozone_user.* FROM email_list_subscriber, ozone_user WHERE ".
            "email_list_subscriber.list_id = '{$list->getListId()}' AND email_list_subscriber.user_id = ozone_user.user_id " .
            "ORDER BY ozone_user.nick_name";

        $c = new Criteria();
        $c->setExplicitQuery($q);

        $users = OzoneUserPeer::instance()->select($c);

        $runData->contextAdd('users', $users);

        $runData->contextAdd('list', $list);
        $runData->contextAdd('site', $site);
    }
}
