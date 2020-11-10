<?php
use DB\PageRevisionPeer;

class WatchedPageChangesFeed extends FeedScreen
{

    protected $requiresAuthentication = true;

    public function render($runData)
    {
        $user = $runData->getTemp("user");
        $key = "watchedpagechanges..".$user->getUserId();
        $mc = OZONE::$memcache;
        $out = $mc->get($key);
        if ($out) {
            return $out;
        }
        $out = parent::render($runData);
        $mc->set($key, $out, 0, 600);
        return $out;
    }

    public function build($runData)
    {

        $user = $runData->getTemp("user");
        $userId = $user->getUserId();

        // set language for the user
        $lang = $user->getLanguage();
        $runData->setLanguage($lang);
        $GLOBALS['lang'] = $lang;

        // and for gettext too:

        switch ($lang) {
            case 'pl':
                $glang="pl_PL";
                break;
            case 'en':
                $glang="en_US";
                break;
        }

        putenv("LANG=$glang");
        putenv("LANGUAGE=$glang");
        setlocale(LC_ALL, $glang.'.UTF-8');

        // now just get watched page changes for the user...

        $c = new Criteria();

        $c->addJoin("page_id", "page.page_id");
        $c->addJoin("page_id", "watched_page.page_id");
        $c->addJoin("user_id", "ozone_user.user_id");
        $c->add("watched_page.user_id", $user->getUserId());
        $c->addOrderDescending("page_revision.revision_id");
        $c->setLimit(30);

        $revisions = PageRevisionPeer::instance()->select($c);

        $channel['title'] = _('Wikijump.com watched pages changes for user').' "'.$user->getNickName().'"';
        $channel['link'] = GlobalProperties::$HTTP_SCHEMA . "://" . GlobalProperties::$URL_HOST . "/account:you/start/watched-changes";

        $items = array();

        foreach ($revisions as $rev) {
            $page = $rev->getPage();
            $site = $page->getSite();
            $item = array();

            $item['title'] = '"'.$page->getTitleOrUnixName().'" '._('on site').' "'.
                $site->getName().'"';
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

            $desc .= _('Site').': <a href="'.GlobalProperties::$HTTP_SCHEMA . "://" . $site->getDomain().'">'.htmlspecialchars($site->getName()).'</a><br/>';
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

            $item['guid'] = $channel['link']."#revision-".$rev->getRevisionId();
            $item['date'] = date('r', $rev->getDateLastEdited()->getTimestamp());

            $content =  '';

            $items[] = $item;
        }

        $runData->contextAdd("channel", $channel);
        $runData->contextAdd("items", $items);
    }
}
