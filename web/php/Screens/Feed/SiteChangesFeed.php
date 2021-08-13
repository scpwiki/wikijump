<?php

namespace Wikidot\Screens\Feed;

use Illuminate\Support\Facades\Cache;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Ozone;
use Wikidot\DB\PageRevisionPeer;
use Wikidot\Utils\FeedScreen;
use Wikidot\Utils\GlobalProperties;
use Wikidot\Utils\WDRenderUtils;

class SiteChangesFeed extends FeedScreen
{

    public function render($runData)
    {
        $site = $runData->getTemp("site");
        $key = "sitechangesfeed..".$site->getSiteId();

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

        $c = new Criteria();

        $c->addJoin("page_id", "page.page_id");
        $c->addJoin("user_id", "users.id");
        $c->add("page.site_id", $site->getSiteId());
        $c->addOrderDescending("page_revision.revision_id");
        $c->setLimit(30);

        $revisions = PageRevisionPeer::instance()->select($c);

        $channel['title'] = _('Recent page changes from site').' "'.htmlspecialchars($site->getName()).'" (a Wikijump site)';
        $channel['link'] = GlobalProperties::$HTTP_SCHEMA . "://" . $site->getDomain();

        $items = array();

        foreach ($revisions as $rev) {
            $page = $rev->getPage();

            $item = array();

            $item['link'] = GlobalProperties::$HTTP_SCHEMA . "://" . $site->getDomain().'/'.$page->getUnixName();

            $desc = '';

            $flags = array();
            if ($rev->getFlagText()) {
                $flags[] = _("source change");
            }
            if ($rev->getFlagTitle()) {
                $flags[] = _("title change");
            }
            if ($rev->getFlagFile()) {
                $flags[] = _("file action");
            }
            if ($rev->getFlagRename()) {
                $flags[] = _("page move/rename");
            }
            if ($rev->getFlagMeta()) {
                $flags[] = _("metadata changed");
            }
            if ($rev->getFlagNew()) {
                $flags[] = _("new page");
            }

            $item['title'] = '"'.$page->getTitleOrUnixName().'" - '.implode(', ', $flags);
            $desc = '';
            $desc .= _('Page').': <a href="'.GlobalProperties::$HTTP_SCHEMA . "://" . $site->getDomain().'/'.$page->getUnixName().'">'.htmlspecialchars($page->getTitle()).'</a> ('.$page->getUnixName().')<br/>';
            $desc .= _('Current revision number').': '.$rev->getRevisionNumber().'<br/>';
            $desc .= _('Date changed').': '.date('r', $rev->getDateLastEdited()->getTimestamp()).'<br/>';
            $desc .= _('Change type').': '.implode(', ', $flags).'<br/>';
            if ($rev->getComments()) {
                $desc .= _('Change comments').': '.htmlspecialchars($rev->getComments()).'<br/>';
            }
            $desc .= _('By').': '.WDRenderUtils::renderUser($rev->getUserOrString()).'<br/>';

            $desc .= '<br/>'._('Page content preview').': <br/>'.$page->getPreview();
            $item['description'] = $desc;

            $item['content'] = $desc;

            $item['guid'] = $item['link']."#revision-".$rev->getRevisionId();
            $item['date'] = date('r', $rev->getDateLastEdited()->getTimestamp());

            $content =  '';

            $items[] = $item;
        }

        $runData->contextAdd("channel", $channel);
        $runData->contextAdd("items", $items);
    }
}
