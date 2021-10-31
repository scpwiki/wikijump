<?php

namespace Wikidot\Modules\List;


use Illuminate\Support\Facades\Cache;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\ODate;
use Ozone\Framework\Ozone;
use Wikidot\DB\CategoryPeer;
use Wikidot\DB\PagePeer;
use Wikidot\DB\PageRevisionPeer;
use Wikidot\DB\ForumThreadPeer;
use Ozone\Framework\SmartyModule;
use Wikidot\Utils\GlobalProperties;
use Wikidot\Utils\ProcessException;
use Wikijump\Helpers\LegacyTools;
use Wikijump\Models\User;
use Wikijump\Services\Wikitext\ParseRenderMode;
use Wikijump\Services\Wikitext\WikitextBackend;

class ListPagesModule extends SmartyModule
{

    public $parameterhash;

    protected $processPage = true;

    private $_tmpSplitSource;
    private $_tmpSource;
    private $_tmpPage;

    private $_vars = array();

    private $_pl;

    private $_parameterUrlPrefix = null;

    public function render($runData)
    {
        $site = $runData->getTemp("site");
        $pl = $runData->getParameterList();
        $this->_pl = $pl;

        $this->_parameterUrlPrefix = $pl->getParameterValue('urlAttrPrefix');
        /*
         * Read all parameters.
         */

        $categoryName = $this->_readParameter(array('category', 'categories'), false);

        $categoryName = strtolower($categoryName);

        $parmHash = md5(serialize($pl->asArrayAll()));
        $this->parameterhash = $parmHash;
        /* Check if recursive. */
        foreach ($this->_moduleChain as $m) {
            if (get_class($m) == 'ListPagesModule') {
                return '<div class="error-block">The ListPages module does not work recursively.</div>';
            }
        }

        $valid = true;

        if (!$categoryName) {
            /* No category name specified, use the current category! */
            $pageUnixName = $runData->getTemp('pageUnixName');
            if (!$pageUnixName) {
                $pageUnixName = $pl->getParameterValue('page_unix_name'); // from preview
            }
            if (strpos($pageUnixName, ":") != false) {
                $tmp0 = explode(':', $pageUnixName);
                $categoryName = $tmp0[0];
            } else {
                $categoryName = "_default";
            }
        }

        $key = 'listpages_v..' . $site->getUnixName() . '..' . $categoryName . '..' . $parmHash;

        $struct = Cache::get($key);
        if (!$struct) {
            $valid = false;
        }
        $cacheTimestamp = $struct['timestamp'];
        $now = time();

        // now check lc for ALL categories involved
        $cats = preg_split('/[,;\s]+?/', $categoryName);

        if ($categoryName != '*') {
            foreach ($cats as $cat) {
                $tkey = 'pagecategory_lc..' . $site->getUnixName() . '..' . $cat; // last change timestamp
                $changeTimestamp = Cache::get($tkey);
                if ($changeTimestamp && $cacheTimestamp && $changeTimestamp <= $cacheTimestamp) {    //cache valid
                } else {
                    $valid = false;
                    if (!$changeTimestamp) {
                        // put timestamp
                        Cache::put($tkey, $now, 864000);
                        $valid = false;
                    }
                }
            }
        } else {
            $akey = 'pageall_lc..' . $site->getUnixName();
            $allPagesTimestamp = Cache::get($akey);
            if ($allPagesTimestamp && $cacheTimestamp && $allPagesTimestamp <= $cacheTimestamp) {    //cache valid
            } else {
                $valid = false;
                if (!$allPagesTimestamp) {
                    // put timestamp
                    Cache::put($akey, $now, 864000);
                    $valid = false;
                }
            }
        }

        if ($valid) {
            $this->_vars = $struct['vars'];
            return $struct['content'];
        }

        $out = parent::render($runData);

        // and store the data now
        $struct = array();
        $now = time();
        $struct['timestamp'] = $now;
        $struct['content'] = $out;
        $struct['vars'] = $this->_vars;

        Cache::put($key, $struct, 864000);
        return $out;
    }

    public function build($runData)
    {
        $pl = $runData->getParameterList();
        $site = $runData->getTemp("site");

        $categoryName = $this->_readParameter(array('category', 'categories'), false);
        $categoryName = strtolower($categoryName);

        $order = $this->_readParameter("order", true);
        $limit = $this->_readParameter("limit", true);
        $perPage = $this->_readParameter("perPage", true);
        $skipCurrent = $this->_readParameter('skipCurrent');

        if ($skipCurrent && ($skipCurrent == 'yes' || $skipCurrent == 'true')) {
            $skipCurrent = true;
        } else {
            $skipCurrent = false;
        }

        $pageUnixName = $runData->getTemp('pageUnixName');
        if (!$pageUnixName) {
            $pageUnixName = $pl->getParameterValue('page_unix_name'); // from preview
        }

        $thisPage = PagePeer::instance()->selectByName($site->getSiteId(), $pageUnixName);
        $pageInfo = PageInfo::fromPageObject($thisPage);

        $categories = array();
        $categoryNames = array();
        if ($categoryName != '*') {
            if (!$categoryName) {
                /* No category name specified, use the current category! */

                if (strpos($pageUnixName, ":") != false) {
                    $tmp0 = explode(':', $pageUnixName);
                    $categoryName = $tmp0[0];
                } else {
                    $categoryName = "_default";
                }
            }
            foreach (preg_split('/[,;\s]+?/', $categoryName) as $cn) {
                $category = CategoryPeer::instance()->selectByName($cn, $site->getSiteId());
                if ($category) {
                    $categories[] = $category;
                    $categoryNames[] = $category->getName();
                }
            }
            if (count($categories) == 0) {
                throw new ProcessException('The requested categories do not (yet) exist.');
            }
        }

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

        /* Handle magic previousBy/nextBy keywords */
        $previousBy = $this->_readParameter('previousBy', true);
        $nextBy = $this->_readParameter('nextBy', true);

        if ($previousBy || $nextBy) {
            if ($refPage = $runData->getTemp('page')) {
                $refPageId = $refPage->getPageId();
                $refPageTitle = $refPage->getTitle() . ' ... ' . $refPage->getUnixName();

                if ($previousBy == 'page_id') {
                    $c->add('page_id', $refPageId, '<');
                } elseif ($nextBy == 'page_id') {
                    $c->add('page_id', $refPageId, '>');
                } elseif ($previousBy == 'title') {
                    $c->add("title || ' ... ' || unix_name", $refPageTitle, '<');
                } elseif ($nextBy == 'title') {
                    $c->add("title || ' ... ' || unix_name", $refPageTitle, '>');
                }
            } else {
                $c->add('page_id', 0); // this should be simply never;
            }
        }

        /* Handle tags! */

        $tagString = $this->_readParameter(['tag', 'tags'], true);

        if ($tagString) {
            /* Split tags. */
            $tags = preg_split('/[\s,\;]+/', $tagString);

            $tagsAny = array();
            $tagsAll = array();
            $tagsNone = array();

            foreach ($tags as $t) {
                if (substr($t, 0, 1) == '+') {
                    $tagsAll[] = substr($t, 1);
                } elseif (substr($t, 0, 1) == '-') {
                    $tagsNone[] = substr($t, 1);
                } elseif ($t == '=') {
                    /* It means: any tags of the current page. */
                    if ($runData->getTemp('page')) {
                        $pageId = $runData->getTemp('page')->getPageId();
                        $tagAny = PagePeer::getTags($pageId);
                        if (count($tagsAny) == 0) {
                            /*
                             * If someone uses the '=' tag, the line below guarantees that
                             * only pages that DO have tags and share at least one similar tag with the
                             * current page are listed.
                             */
                            $tagsAny[] = '   ';
                        }
                    }
                } else {
                    $tagsAny[] = $t;
                }
            }

            /*
             * One more condition: if $tagString is equal to "=" only (which means "similar pages by tags),
             * it is reasonable to drop current page from being displayed.
             */
            if ($tagString == '=') {
                $skipCurrent = true;
            }

            /* Create Extra conditions to the SELECT */

            /* ANY */
            if (count($tagsAny) > 0) {
                $t = array();
                foreach ($tagsAny as $tag0) {
                    $t[] = 'tag = \'' . db_escape_string($tag0) . '\'';
                }
                $tagQuery = "SELECT count(*) FROM page " . "WHERE (" . implode(' OR ', $t) . ")";

                $c->add('(' . $tagQuery . ')', 1, '>=');
            }
            /* ALL */
            if (count($tagsAll) > 0) {
                $t = array();
                foreach ($tagsAll as $tag0) {
                    $t[] = 'tag = \'' . db_escape_string($tag0) . '\'';
                }
                $tagQuery = "SELECT count(*) FROM page " . "WHERE (" . implode(' OR ', $t) . ")";

                $c->add('(' . $tagQuery . ')', count($tagsAll));
            }
            /* NONE */
            if (count($tagsNone) > 0) {
                $t = array();
                foreach ($tagsNone as $tag0) {
                    $t[] = 'tag = \'' . db_escape_string($tag0) . '\'';
                }
                $tagQuery = "SELECT count(*) FROM page " . "WHERE (" . implode(' OR ', $t) . ")";

                $c->add('(' . $tagQuery . ')', 0);
            }
        }

        if ($skipCurrent && $runData->getTemp('page') && $runData->getTemp('page')->getPageId()) {
            $c->add('page_id', $runData->getTemp('page')->getPageId(), '!=');
        }
        /* Handle date ranges. */

        $date = $this->_readParameter("date", true);

        $dateA = array();
        if (preg_match('/^[0-9]{4}$/', $date)) {
            $dateA['year'] = $date;
        }
        if (preg_match('/^[0-9]{4}\.[0-9]{1,2}$/', $date)) {
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

        /* Handle date "last X day(s)/week(s)/month(s)" */

        $m = array();
        if (preg_match('/^last (?:([1-9][0-9]*) )?(day|week|month)s?$/', $date, $m)) {
            $dateObj = new ODate();
            $n = $m[1];
            if (!$n) {
                $n = 1;
            }
            $unit = $m[2];
            $convarray = array('day' => 86400, 'week' => 604800, 'month' => 2592000);
            $dateObj->subtractSeconds($n * $convarray[$unit]);
            $c->add('date_created', $dateObj, '>');
        }

        /* Handle pagination. */

        if (!$perPage || !preg_match('/^[0-9]+$/', $perPage)) {
            $perPage = 20;
        }

        if ($limit && preg_match('/^[0-9]+$/', $limit)) {
            $c->setLimit($limit); // this limit has no effect on count(*) !!!
        } else {
            $limit = null;
        }

        $pageNo = $pl->getParameterValue(($this->_parameterUrlPrefix ? ($this->_parameterUrlPrefix . '_') : '' ) . "p");
        if ($pageNo == null || !preg_match('/^[0-9]+$/', $pageNo)) {
            $pageNo = 1;
        }

        $co = PagePeer::instance()->selectCount($c);

        if ($limit) {
            $co = min(array($co, $limit));
        }

        $totalPages = ceil($co / $perPage);
        if ($pageNo > $totalPages) {
            $pageNo = $totalPages;
        }
        $offset = ($pageNo - 1) * $perPage;
        if ($limit) {
            $newLimit = min(array($perPage, $limit - $offset));
        } else {
            $newLimit = $perPage;
        }
        $c->setLimit($newLimit, $offset);
        $runData->contextAdd("totalPages", $totalPages);
        $runData->contextAdd("currentPage", $pageNo);
        $runData->contextAdd("count", $co);
        $runData->contextAdd("totalPages", $totalPages);
        $runData->contextAdd('parameterUrlPrefix', $this->_parameterUrlPrefix);

        /* Pager's base url */
        $url = $_SERVER['REQUEST_URI'];
        if (($url == '' || $url == '/') && isset($pageUnixName)) {
            $url = '/' . $pageUnixName;
        }
        $pref = '';
        if ($this->_parameterUrlPrefix) {
            $pref = $this->_parameterUrlPrefix . '_';
        }
        $url = preg_replace(';(/'.$pref.'p/[0-9]+)|$;', '/'.$pref.'p/%d', $url, 1);
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
            case 'ratingAsc':
                $c->addOrderAscending('rate');
                break;
            case 'ratingDesc':
                $c->addOrderDescending('rate');
                break;
            default:
            case 'dateCreatedDesc':
                $c->addOrderDescending('page_id');
                break;
        }

        $pages = PagePeer::instance()->select($c);

        /* Process... */
        $format = $this->_readParameter("module_body");
        if (!$format) {
            $format = "" . "+ %%linked_title%%\n\n" . _("by") . " %%author%% %%date|%O ago (%e %b %Y, %H:%M %Z)%%\n\n" . "%%short%%";
        }

        $items = array();

        $separation = $this->_readParameter("separate");
        if ($separation == 'no' || $separation == 'false') {
            $separation = false;
        } else {
            $separation = true;
        }

        foreach ($pages as $page) {
            $this->_tmpPage = $page;
            $title = $page->getTitle();
            $source = $page->getSource();

            $title = str_replace(array('[',']'), '', $title);
            $title = str_replace('%%', "\xFD", $title);

            $source = str_replace('%%', "\xFD", $source);

            $c = new Criteria();
            $c->add('revision_id', $page->getRevisionId());
            $lastRevision = PageRevisionPeer::instance()->selectOne($c);

            //$c = new Criteria();
            //$c->add('page_id', $page->getPageId());
            //$c->addOrderAscending('revision_id');
            //$firstRevision = Wikidot_DB_PageRevisionPeer::instance()->selectOne($c);
            $b = $format;

            /* A series of substitutions. */

            $b = str_replace("\xFD", '', $b);

            /* %%title%% and similar */

            $b = str_replace('%%title%%', $title, $b);
            $b = preg_replace("/%%((linked_title)|(title_linked))%%/i", preg_quote_replacement('[[[' . $page->getUnixName() . ' | ' . $title . ']]]'), $b);

            /* %%author%% */

            if ($page->getOwnerUserId()) {
                $user = User::find($page->getOwnerUserId());
                if (LegacyTools::isSystemAccount($user->getUserId()) === false) {
                    $userString = '[[*user ' . $user->username . ']]';
                } else {
                    $userString = _('Anonymous user');
                }
            } else {
                $userString = _('Anonymous user');
            }
            $b = str_ireplace("%%author%%", $userString, $b);
            $b = str_ireplace("%%user%%", $userString, $b);

            if ($lastRevision->getUserId()) {
                $user = User::find($lastRevision->getUserId());
                if (LegacyTools::isSystemAccount($user->id) === false) {
                    $userString = '[[*user ' . $user->username . ']]';
                } else {
                    $userString = _('Anonymous user');
                }
            } else {
                $userString = _('Anonymous user');
            }
            $b = str_ireplace("%%author_edited%%", $userString, $b);
            $b = str_ireplace("%%user_edited%%", $userString, $b);

            /* %%date%% */

            $b = preg_replace(';%%date(\|.*?)?%%;', '%%date|' . $page->getDateCreated()->getTimestamp() . '\\1%%', $b);
            $b = preg_replace(';%%date_edited(\|.*?)?%%;', '%%date|' . $page->getDateLastEdited()->getTimestamp() . '\\1%%', $b);

            /* %%content%% */

            $b = preg_replace(';%%((body)|(text)|(long)|(content))%%;i', $source, $b);

            /* %%content{n}%% */

            /* Split the content first. */
            $this->_tmpSplitSource = preg_split('/^([=]{4,})$/m', $source);
            $this->_tmpSource = $source;
            $b = preg_replace_callback('/%%content{([0-9]+)}%%/', array(
                $this,
                '_handleContentSubstitution'), $b);

            /* %%short%% */

            $b = preg_replace_callback("/%%((description)|(short)|(summary))%%/i", array(
                $this,
                '_handleSummary'), $b);

            $b = preg_replace_callback("/%%first_paragraph%%/i", array(
                $this,
                '_handleFirstParagraph'), $b);

            /* %%preview%% */
            $b = preg_replace_callback("/%%preview(?:\(([0-9]+)\))?%%/i", array(
                $this,
                '_handlePreview'), $b);

            /* %%rating%% */
            $b = str_ireplace('%%rating%%', $page->getRate(), $b);

            /* %%comments%% */
            $b = preg_replace_callback("/%%comments%%/i", array(
                $this, '_handleCommentsCount'), $b);

            /* %%page_unix_name%% */
            $b = str_ireplace('%%page_unix_name%%', $page->getUnixName(), $b);

            /* %%category%% */

            if (strpos($page->getUnixName(), ":") != false) {
                $tmp0 = explode(':', $page->getUnixName());
                $categoryName00 = $tmp0[0];
            } else {
                $categoryName00 = "_default";
            }

            $b = str_ireplace('%%category%%', $categoryName00, $b);

            /* %%link%% */
            $b = str_ireplace('%%link%%', GlobalProperties::$HTTP_SCHEMA . "://" . $site->getDomain().'/'.$page->getUnixName(), $b);

            /* %%tags%% */
            $b = preg_replace_callback("/%%tags%%/i", array(
                $this, '_handleTags'), $b);

            $b = str_replace("\xFD", '%%', $b);

            if ($separation) {
                $wt = WikitextBackend::make(ParseRenderMode::LIST, $pageInfo);
                $b = $wt->renderHtml($b)->body;
                $b = "<div class=\"list-pages-item\">\n" . $b . "</div>";
            }

            $items[] = trim($b);
        }
        if (!$separation) {
            $prependLine = $this->_readParameter('prependLine');
            $appendLine = $this->_readParameter('appendLine');

            $prefix = $prependLine ? ($prependLine . "\n") : '';
            $suffix = $appendLine ? ("\n" . $appendLine) : '';

            $modifiedSource = $prefix . implode("\n", $items) . $suffix;

            $wt = WikitextBackend::make(ParseRenderMode::LIST, $pageInfo);
            $itemsContent = $wt->renderHtml($modifiedSource)->body;
        } else {
            $itemsContent = implode("\n", $items);
        }
        /*
         * If separation is false, we are not separating the items with double-newlines but rather
         * with a single newline. This allows to create e.g. list of pages by creating a template:
         * * %%linked_title%%
         */

        /* Fix dates. */
        //$dateString = '<span class="odate">'.$thread->getDateStarted()->getTimestamp().'|%e %b %Y, %H:%M %Z|agohover</span>';
        $itemsContent = preg_replace_callback('/%%date\|([0-9]+)(\|.*?)?%%/', array(
            $this, '_formatDate'), $itemsContent);

        $runData->contextAdd("items", $items);
        $runData->contextAdd("itemsContent", $itemsContent);
        $runData->contextAdd("details", $details);
        $runData->contextAdd("preview", $preview);

        /* Also build an URL for the feed. */

        $rssTitle = $this->_readParameter(array('rss', 'rssTitle'));

        if ($rssTitle !== null) {
            $url = GlobalProperties::$HTTP_SCHEMA . "://" . $site->getDomain() . '/feed/pages';
            if (count($categoryNames) > 0) {
                $url .= '/category/' . urlencode(implode(',', $categoryNames));
            }
            if (isset($tags)) {
                $url .= '/tags/' . urlencode(implode(',', $tags));
            }

            /*
             * Ignore date in RSS generation.
             */
            /*
            if (isset($date)) {
                $url .= '/date/' . urlencode($date);
            }*/

            if ($order) {
                $url .= '/order/' . urlencode($order);
            }

            //$erss = $pl->getParameterValue('rssEmbed');
            //if ($erss == 'no' || $erss == 'false') {
            //    $erss = false;
            //} else {
            //    $erss = true;
            //}
            //$srss = $pl->getParameterValue('rssShow');
            //if ($srss == 'no' || $srss == 'false') {
            //    $srss = false;
            //} else {
            //    $srss = true;
            //}

            //$trss = $pl->getParameterValue('rssTitle');
            //if ($trss) {
            //    $url .= '/t/' . urlencode($trss);
            //}
            $url .= '/t/'  . urlencode($rssTitle);
            //if ($erss) {
            $this->_vars['rssUrl'] = $url;
            $this->_vars['rssTitle'] = $rssTitle;
            //}
            //if ($srss) {
            $runData->contextAdd('rssUrl', $url);
            $runData->contextAdd('rssTitle', $rssTitle);
            //}
        }
    }

    private function _formatDate($m)
    {
        if (isset($m[2])) {
            $format = $m[2];
        } else {
            $format = '|%e %b %Y, %H:%M %Z|agohover';
        }
        $dateString = '<span class="odate">' . $m[1] . $format . '</span>';
        return $dateString;
    }

    private function _handleContentSubstitution($m)
    {
        $n = $m[1] - 1;

        if (isset($this->_tmpSplitSource[$n])) {
            return trim($this->_tmpSplitSource[$n]);
        } else {
            return '';
        }
    }

    private function _handleSummary($m)
    {
        if (isset($this->_tmpSplitSource[0]) && count($this->_tmpSplitSource) > 1) {
            return trim($this->_tmpSplitSource[0]);
        } else {
            /* Try to extract the short version. */
            $s = $this->_tmpSource;
            /* Strip some blocks first. */
            $s = trim(preg_replace('/^(\+{1,6}) (.*)/m', "\n\n", $s));
            $s = trim(preg_replace('/^\[\[toc(\s[^\]]+)?\]\]/', "\n\n", $s));
            $s = trim(preg_replace('/^\[\[\/?div(\s[^\]]+)?\]\]/', "\n\n", $s));
            $s = trim(preg_replace('/^\[\[\/?module(\s[^\]]+)?\]\]/', "\n\n", $s));
            /* 1. Try the first paragraph. */
            $m1 = array();
            $split = preg_split("/\n{2,}/", $s);
            //var_dump($split);
            return trim($split[0]);
        }
    }

    private function _handleFirstParagraph($m)
    {
        /* Try to extract the short version. */
        $s = $this->_tmpSource;
        /* Strip some blocks first. */
        $s = trim(preg_replace('/^(\+{1,6}) (.*)/m', "\n\n", $s));
        $s = trim(preg_replace('/^\[\[toc(\s[^\]]+)?\]\]/', "\n\n", $s));
        $s = trim(preg_replace('/^\[\[\/?div(\s[^\]]+)?\]\]/', "\n\n", $s));
        $s = trim(preg_replace('/^\[\[\/?module(\s[^\]]+)?\]\]/', "\n\n", $s));
        /* 1. Try the first paragraph. */
        $m1 = array();
        $split = preg_split("/\n{2,}/", $s);
        //var_dump($split);
        return trim($split[0]);
    }

    private function _handleTags($m)
    {
        $page = $this->_tmpPage;
        /* Select tags. */
        // get the tags
        $t2 = PagePeer::getTags($pageId);
        if (count($t2) == 0) {
            return _('//no tags found for this page//');
        }
        $tagTarget = $this->_readParameter('tagTarget', true);
        if ($tagTarget) {
            $t3 = array();
            $p = 'tag';
            if ($this->_parameterUrlPrefix) {
                $p = $this->_parameterUrlPrefix . '_tag';
            }
            foreach ($t2 as $t) {
                $t3[] = '[/'.$tagTarget.'/'.$p.'/'.urlencode($t).' '.$t.']';
            }
            return implode(' ', $t3);
        }
        return implode(' ', $t2);
    }

    private function _handlePreview($m)
    {
        $page = $this->_tmpPage;
        $length = 200;
        if (isset($m[1])) {
            $length = $m[1];
        }

        return $page->getPreview($length);
    }

    private function _handleCommentsCount($m)
    {
        $page = $this->_tmpPage;
        $threadId = $page->getThreadId();
        if ($threadId) {
            $thread = ForumThreadPeer::instance()->selectByPrimaryKey($threadId);
            if ($thread) {
                return $thread->getNumberPosts();
            }
        }
        return 0;
    }

    protected function _readParameter($name, $fromUrl = false)
    {
        $pl = $this->_pl;
        $name = (array) $name;
        foreach ($name as $n) {
            $val = $pl->getParameterValue($n, "MODULE", "AMODULE");
            if ($val) {
                break;
            }
        }
        if ($fromUrl && $val == '@URL') {
            foreach ($name as $n) {
                if ($this->_parameterUrlPrefix) {
                    $n = $this->_parameterUrlPrefix . '_' . $n;
                }
                $val = $pl->resolveParameter($n, 'GET');
                if ($val) {
                    break;
                }
            }
        }

        return $val;
    }

    public function processPage($out, $runData)
    {
        $pl = $runData->getParameterList();
        $pl->getParameterValue('t');
        if ($this->_vars['rssUrl']) {
            $rssTitle = $this->_vars['rssTitle'];
            if (!$rssTitle) {
                $rssTitle = _('Page RSS Feed');
            }
            $out = preg_replace("/<\/head>/", '<link rel="alternate" type="application/rss+xml" title="' . preg_quote_replacement(htmlspecialchars($rssTitle)) . '" href="' . $this->_vars['rssUrl'] . '"/></head>', $out, 1);
        }
        return $out;
    }
}
