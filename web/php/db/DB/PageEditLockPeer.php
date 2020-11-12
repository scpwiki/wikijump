<?php
namespace DB;

use Criteria;
use ODate;

/**
 * Object Model class.
 *
 */
class PageEditLockPeer extends PageEditLockPeerBase
{

    public function deleteOutdated($pageId)
    {
        $c = new Criteria();
        $c->add("page_id", $pageId);
        $d = new ODate();
        $c->add("date_last_accessed", $d->addSeconds(-15 * 60), '<');
        $this->delete($c);
    }

    public function deleteOutdatedByPageName($siteId, $pageName)
    {
        $c = new Criteria();
        $c->add("page_unix_name", $pageName);
        $c->add("site_id", $siteId);
        $d = new ODate();
        $c->add("date_last_accessed", $d->addSeconds(-15 * 60), '<');
        $this->delete($c);
    }
}
