<?php

namespace Wikidot\Screens\Feed;

use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Ozone;
use Wikidot\DB\CategoryPeer;
use Wikidot\DB\PagePeer;
use Wikidot\Utils\FeedScreen;
use Wikidot\Utils\GlobalProperties;
use Wikijump\Models\User;
use Wikijump\Services\Wikitext\ParseRenderMode;
use Wikijump\Services\Wikitext\WikitextBackend;

class PagesFeed extends FeedScreen
{

    public function render($runData)
    {
        $site = $runData->getTemp("site");
        $pl = $runData->getParameterList();
        $categoryName = $pl->getParameterValue("category");
        $parmHash = md5(serialize($pl->asArray()));

        $key = 'listpagesfeed_v..'.$site->getUnixName().'..'.$categoryName.'..'.$parmHash;

        $valid = true;

        $mc = OZONE::$memcache;
        $struct = $mc->get($key);
        if (!$struct) {
            $valid = false;
        }
        $cacheTimestamp = $struct['timestamp'];
        $now = time();

        // now check lc for ALL categories involved
        $cats = preg_split('/[,;\s]+?/', $categoryName);

        foreach ($cats as $cat) {
            $tkey = 'pagecategory_lc..'.$site->getUnixName().'..'.$cat; // last change timestamp
            $changeTimestamp = $mc->get($tkey);
            if ($changeTimestamp && $cacheTimestamp && $changeTimestamp <= $cacheTimestamp) {
                //cache valid
            } else {
                $valid = false;
                if (!$changeTimestamp) {
                    //  put timestamp
                    $mc->set($tkey, $now, 0, 864000);
                    $valid = false;
                }
            }
        }

        if (count($cats) == 0) {
            $akey = 'pageall_lc..'.$site->getUnixName();
            $allPagesTimestamp = $mc->get($akey);
            if ($allPagesTimestamp && $cacheTimestamp && $allPagesTimestamp <= $cacheTimestamp) {
                //cache valid
            } else {
                $valid = false;
                if (!$allPagesTimestamp) {
                    //  put timestamp
                    $mc->set($akey, $now, 0, 864000);
                    $valid = false;
                }
            }
        }

        if ($valid) {
            $this->vars = $struct['vars'];
            return $struct['content'];
        }

        $out = parent::render($runData);

        // and store the data now
        $struct = array();
        $now = time();
        $struct['timestamp'] = $now;
        $struct['content'] = $out;
        $struct['vars']=$this->vars;

        $mc->set($key, $struct, 0, 864000);

        return $out;
    }

    public function build($runData)
    {

        $pl = $runData->getParameterList();
        $site = $runData->getTemp("site");

        $categoryName = $pl->getParameterValue("category");

        $order = $pl->getParameterValue("order");
        $limit = $pl->getParameterValue("limit");
        $perPage = $pl->getParameterValue("perPage");

        $categories = array();
        $categoryNames = array();

        foreach (preg_split('/[,;\s]+?/', $categoryName) as $cn) {
            $category = CategoryPeer::instance()->selectByName($cn, $site->getSiteId());
            if ($category) {
                $categories[] = $category;
                $categoryNames[] = $category->getName();
            }
        }
        //if(count($categories) == 0){
        //  throw new ProcessException(_("The category cannot be found."));
        //}


        // now select pages according to the specified criteria

        $c = new Criteria();
        $c->add("site_id", $site->getSiteId());
        if (count($categories) > 0) {
            $ccat = new Criteria();
            foreach ($categories as $cat) {
                $ccat->addOr('category_id', $cat->getCategoryId());
            }
            $c->addCriteriaAnd($ccat);
        }

        $c->add('unix_name', '(^|:)_', '!~');

        /* Handle tags! */

        $tagString = $pl->getParameterValue("tag");
        if (!$tagString) {
            $tagString = $pl->getParameterValue("tags");
        }

        if ($tagString) {
            /* Split tags. */
            $tags = preg_split(';[\s,\;]+;', $tagString);

            $tagsAny = array();
            $tagsAll = array();
            $tagsNone = array();

            foreach ($tags as $t) {
                if (substr($t, 0, 1) == '+') {
                    $tagsAll[] = substr($t, 1);
                } elseif (substr($t, 0, 1) == '-') {
                    $tagsNone[] = substr($t, 1);
                } else {
                    $tagsAny[] = $t;
                }
            }

            /* Create Extra conditions to the SELECT */

            /* ANY */
            if (count($tagsAny) > 0) {
                $t = array();
                foreach ($tagsAny as $tag0) {
                    $t[] = 'tag = \''.db_escape_string($tag0).'\'';
                }
                $tagQuery = "SELECT count(*) FROM page_tag "
                    ."WHERE page_tag.page_id=page.page_id "
                    ."AND (".implode(' OR ', $t).")";

                $c->add('('.$tagQuery.')', 1, '>=');
            }
            /* ALL */
            if (count($tagsAll) > 0) {
                $t = array();
                foreach ($tagsAll as $tag0) {
                    $t[] = 'tag = \''.db_escape_string($tag0).'\'';
                }
                $tagQuery = "SELECT count(*) FROM page_tag "
                    ."WHERE page_tag.page_id=page.page_id "
                    ."AND (".implode(' OR ', $t).")";

                $c->add('('.$tagQuery.')', count($tagsAll));
            }
            /* NONE */
            if (count($tagsNone) > 0) {
                $t = array();
                foreach ($tagsNone as $tag0) {
                    $t[] = 'tag = \''.db_escape_string($tag0).'\'';
                }
                $tagQuery = "SELECT count(*) FROM page_tag "
                    ."WHERE page_tag.page_id=page.page_id "
                    ."AND (".implode(' OR ', $t).")";

                $c->add('('.$tagQuery.')', 0);
            }
        }

        /* Handle date ranges. */

        $date = $pl->getParameterValue("date");

        $dateA = array();
        if (preg_match(';^[0-9]{4};', $date)) {
            $dateA['year'] = $date;
        }
        if (preg_match(';^[0-9]{4}\.[0-9]{1,2};', $date)) {
            $dateS = explode('.', $date);
            $dateA['year'] = $dateS[0];
            $dateA['month'] = $dateS[1];
        }

        if (isset($dateA['year'])) {
            $c->add('EXTRACT(YEAR FROM date_created)', $dateA['year']);
        }

        if (isset($dateA['month'])) {
            $c->add('EXTRACT(MONTH FROM date_created)', $dateA['month']);
        }

        /* Handle pagination. */

        if (!$perPage || !preg_match(';^[0-9]+$;', $perPage)) {
            $perPage = 20;
        }

        if ($limit && preg_match(';^[0-9]+$;', $perPage)) {
            $c->setLimit($limit);
        }

        $pageNo = $pl->getParameterValue("p");
        if ($pageNo == null || !preg_match(';^[0-9]+$;', $pageNo)) {
            $pageNo = 1;
        }

        $co = PagePeer::instance()->selectCount($c);

        $totalPages = ceil($co/$perPage);
        if ($pageNo>$totalPages) {
            $pageNo = $totalPages;
        }
        $offset = ($pageNo-1) * $perPage;

        $c->setLimit($perPage, $offset);
        $runData->contextAdd("totalPages", $totalPages);
        $runData->contextAdd("currentPage", $pageNo);
        $runData->contextAdd("count", $co);
        $runData->contextAdd("totalPages", $totalPages);

        /* Pager's base url */
        $url = $_SERVER['REQUEST_URI'];
        $url = preg_replace(';(/p/[0-9]+)|$;', '/p/%d', $url, 1);
        $runData->contextAdd("pagerUrl", $url);

        switch ($order) {
            case 'dateCreatedAsc':
                $c->addOrderAscending('page_id');
                break;
            case 'dateEditedDesc':
                $c->addOrderDescending('date_last_edited');
                break;
            case 'dateEditedAsc':
                $c->addOrderAscending('date_last_edited');
                break;
            case 'titleDesc':
                $c->addOrderDescending("COALESCE(title, unix_name)");
                break;
            case 'titleAsc':
                $c->addOrderAscending("COALESCE(title, unix_name)");
                break;
            default:
            case 'dateCreatedDesc':
                $c->addOrderDescending('page_id');
                break;
        }

        $pages = PagePeer::instance()->select($c);

        /* Process... */
        $format = $pl->getParameterValue("module_body");
        if (!$format) {
            $format = "" .
                    "+ %%linked_title%%\n\n" .
                    _("by")." %%author%% %%date|%O ago (%e %b %Y, %H:%M %Z)%%\n\n" .
                    "%%content%%\n\n%%comments%%";
        }

        $items = array();

        foreach ($pages as $page) {
            $title = $page->getTitle();
            $source = $page->getSource();

            $item = array();

            $item['title'] = $page->getTitle();
            $item['link'] = GlobalProperties::$HTTP_SCHEMA . "://" . $site->getDomain()."/".$page->getUnixName();
            $item['guid'] = $item['link'];
            $item['date'] = date('r', $page->getDateCreated()->getTimestamp());


            $b = '';

            /* Create content of the feed. */
            $cont = '';
            /* get summary for the page. */
            $splitSource = preg_split('/^([=]{4,})$/m', $source);
            if (isset($splitSource[0]) && count($splitSource) > 1) {
                $cont = $splitSource[0];
            } else {
                /* Try to extract the short version. */
                $s = $source;
                /* 1. Try the first paragraph. */
                $m1 = array();
                preg_match(";(^.*?)\n\n;", $s, $m1);
                if (isset($m1[1])) {
                    $p = $m1[1];
                    $cont =  $p;
                } else {
                    $cont = $s;
                }
            }

            $b .= $cont ."\n\n";

            /* %%author%% */
            $ownerUserId = $page->getOwnerUserId();
            if ($ownerUserId) {
                $user = User::find($ownerUserId);
                $userString = '[[*user '.$user->username.']]';
            } else {
                $userString = 'Anonymous user';
            }
            $b .= 'by ' . $userString;

            $pageInfo = PageInfo::fromPageObject($page);
            $wt = WikitextBackend::make(ParseRenderMode::LIST, $pageInfo);
            $wt->renderHtml($b)->body;

            $d = utf8_encode("\xFE");
            $content = preg_replace("/" . $d . "module \"([a-zA-Z0-9\/_]+?)\"(.+?)?" . $d . "/", '', $content);
            $content = preg_replace(';(<.*?)(src|href)="/([^"]+)"([^>]*>);si', '\\1\\2="'.GlobalProperties::$HTTP_SCHEMA . "://" . $site->getDomain().'/\\3"\\4', $content);
            $content = preg_replace(';<script\s+[^>]+>.*?</script>;is', '', $content);
            $content = preg_replace(';(<[^>]*\s+)on[a-z]+="[^"]+"([^>]*>);si', '\\1 \\2', $content);

            $item['content'] = $content;

            $items[] = $item;
        }
        $channel = array();
        $channel['title'] = $pl->getParameterValue("t");
        //$channel['link'] = "http://".$site->getDomain()."/".$page->getUnixName();
//      if($feed->getDescription()){
//          $channel['description'] = $feed->getDescription();
//      }

        $runData->contextAdd("channel", $channel);
        $runData->contextAdd("items", $items);
    }
}
