<?php

namespace Wikidot\Jobs;

use Exception;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Database\Database;
use Ozone\Framework\ODate;
use Ozone\Framework\SchedulerJob;
use Wikidot\DB\PageExternalLinkPeer;
use Wikidot\Pingback\Pingback;
use Wikidot\Pingback\PingbackException;

/**
 * Sends pingbacks.
 */
class SendPingbacksJob implements SchedulerJob
{

    public function run()
    {
        Database::init();

        while ($link = $this->_selectLink()) {
            $this->_ping($link);
        }
    }

    protected function _selectLink()
    {
        $date = new ODate();
        $date->subtractSeconds(3600);
        $date0 = new ODate();
        $date0->subtractSeconds(600);

        $q = "SELECT page_external_link.* FROM page_external_link, site_settings, site, category, page" .
             " WHERE page_external_link.pinged = false AND page_external_link.date > '".db_escape_string($date->getDate())."' " .
             " AND page_external_link.date < '".db_escape_string($date0->getDate())."' " .
             " AND (category.enable_pingback_out = true OR site_settings.enable_all_pingback_out = true) " .
             " AND site.private = false AND site.visible=true AND site.deleted=false " .
             " AND page_external_link.page_id = page.page_id" .
             " AND page.category_id = category.category_id AND category.site_id = site_settings.site_id AND page.site_id = site.site_id LIMIT 1";
        $c = new Criteria();
        $c->setExplicitQuery($q);
        $link = PageExternalLinkPeer::instance()->selectOne($c);
        return $link;
    }

    protected function _ping($link)
    {
        $link->setPinged(true);
        $link->setPingStatus('PROCESSING');
        $link->save();

        $h = $link->buildPageUrl();

        $ping = new PingBack($link->getToUrl(), $h);
        try {
            $status = $ping->ping();
            $link->setPingStatus($status);
        } catch (PingBackException $e) {
            $link->setPingStatus($e->getMessage());
        } catch (Exception $e) {
            $link->setPingStatus('EXCEPTION');
        }
        $link->save();
        //echo $h;
    }
}
