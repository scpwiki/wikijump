<?php
use DB\EmailList;
use DB\EmailListPeer;
use DB\EmailListSubscriberPeer;

class ManageSiteEmailListsAction extends SmartyAction
{

    public function isAllowed($runData)
    {
        WDPermissionManager::instance()->hasPermission('manage_site', $runData->getUser(), $runData->getTemp("site"));

        return true;
    }

    public function perform($r)
    {
    }

    public function saveListEvent($runData)
    {

        $pl =  $runData->getParameterList();
        $site = $runData->getTemp("site");
        $siteId = $site->getSiteId();

        $listId = $pl->getParameterValue('listId');
        $isNew = ! (bool) $listId;
        $listTitle = trim($pl->getParameterValue('title'));
        $listUnixName = trim($pl->getParameterValue('unixName'));
        $listWhoCanJoin = trim($pl->getParameterValue('whoCanJoin'));

        if (strlen($listTitle) > 30) {
            throw new ProcessException('List title cannot be longer than 30 characters');
        }
        if (strlen($listTitle) == 0) {
            throw new ProcessException('Title of the list should be provided.');
        }
        if (strlen($listUnixName) > 30) {
            throw new ProcessException('Unix name (address) of the list cannot be longer than 20 characters');
        }
        if (strlen($listUnixName) == 0) {
            throw new ProcessException('Unix name (address) of the list should be provided.');
        }

        $db = Database::connection();
        $db->begin();

        $list = null;
        if ($isNew) {
            $list = new EmailList();
            $list->setSiteId($siteId);
        } else {
            $c = new Criteria();
            $c->add('list_id', $listId);
            $c->add('site_id', $site->getSiteId());
            $list = EmailListPeer::instance()->selectOne($c);
        }
        $list->setTitle($listTitle);
        $list->setUnixName($listUnixName);
        $list->setWhoCanJoin($listWhoCanJoin);

        try {
            $list->save();
        } catch (Exception $e) {
            throw new ProcessException("List cannot be saved.");
        }
        $db->commit();
    }

    public function unsubscribeEvent($runData)
    {
        $pl =  $runData->getParameterList();
        $site = $runData->getTemp("site");
        $siteId = $site->getSiteId();
        $listId = $pl->getParameterValue('listId');
        $userId = $pl->getParameterValue('userId');
        $c = new Criteria();
        $c->add('list_id', $listId);
        $c->add('site_id', $site->getSiteId());

        $db = Database::connection();
        $db->begin();
        $list = EmailListPeer::instance()->selectOne($c);
        if (!$list) {
            throw new ProcessException('The requested list  cannot be found.');
        }

        $c = new Criteria();
        $c->add('list_id', $listId);
        $c->add('user_id', $userId);

        EmailListSubscriberPeer::instance()->delete($c);
        $list->calculateSubscriberCount();
        $list->save();
        $db->commit();
    }
}
