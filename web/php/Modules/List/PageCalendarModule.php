<?php

namespace Wikidot\Modules\List;


use Illuminate\Support\Facades\Cache;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Database\Database;
use Ozone\Framework\Ozone;
use Wikidot\DB\CategoryPeer;
use Wikidot\DB\PageTagPeer;
use Wikidot\DB\PagePeer;

use Ozone\Framework\SmartyModule;
use Wikidot\Utils\ProcessException;

class PageCalendarModule extends SmartyModule
{

    protected $_pl;
    protected $parameterhash;
    protected $_vars;
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

        $key = 'pagecalendar_v..' . $site->getUnixName() . '..' . $categoryName . '..' . $parmHash;

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
                        //  put timestamp
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
                    //  put timestamp
                    Cache::put($akey, $now, 864000);
                    $valid = false;
                }
            }
        }

        if ($valid) {
            $this->_vars = $struct['vars'];
            //echo 'fromcache';
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
        $this->_pl = $pl;
        $site = $runData->getTemp("site");

        $categoryName = $this->_readParameter(array('category', 'categories'), false);

        $categoryName = strtolower($categoryName);

        $startPage = $this->_readParameter(array('startPage', 'targetPage'));

        if (!$startPage) {
            /* Get curent page. */
            $startPage = $runData->getTemp('pageUnixName');
            if (!$startPage) {
                $startPage = $pl->getParameterValue('page_unix_name'); // from preview
            }
        }

        $categories = array();
        $categoryNames = array();
        if ($categoryName != '*') {
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
        //if(count($categories) == 0){
        //  throw new ProcessException(_("The category cannot be found."));
        //}


        $attrUrlPrefix = $pl->getParameterValue('urlAttrPrefix');

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

        $tagString = $this->_readParameter(array('tag', 'tags'), true);

        //var_dump($tagString);

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
                        $co = new Criteria();
                        $co->add("page_id", $pageId);
                        $co->addOrderAscending("tag");
                        $tagso = PageTagPeer::instance()->select($co);
                        foreach ($tagso as $to) {
                            $tagsAny[] = $to->getTag();
                        }
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


            /* ANY */
            if (count($tagsAny) > 0) {
                $t = array();
                foreach ($tagsAny as $tag0) {
                    $t[] = 'tag = \'' . db_escape_string($tag0) . '\'';
                }
                $tagQuery = "SELECT count(*) FROM page_tag " . "WHERE page_tag.page_id=page.page_id " . "AND (" . implode(' OR ', $t) . ")";

                $c->add('(' . $tagQuery . ')', 1, '>=');
            }
            /* ALL */
            if (count($tagsAll) > 0) {
                $t = array();
                foreach ($tagsAll as $tag0) {
                    $t[] = 'tag = \'' . db_escape_string($tag0) . '\'';
                }
                $tagQuery = "SELECT count(*) FROM page_tag " . "WHERE page_tag.page_id=page.page_id " . "AND (" . implode(' OR ', $t) . ")";

                $c->add('(' . $tagQuery . ')', count($tagsAll));
            }
            /* NONE */
            if (count($tagsNone) > 0) {
                $t = array();
                foreach ($tagsNone as $tag0) {
                    $t[] = 'tag = \'' . db_escape_string($tag0) . '\'';
                }
                $tagQuery = "SELECT count(*) FROM page_tag " . "WHERE page_tag.page_id=page.page_id " . "AND (" . implode(' OR ', $t) . ")";

                $c->add('(' . $tagQuery . ')', 0);
            }
        }
        $c->addGroupBy('datestring');

        $db = Database::connection();


        $corig = clone($c);
        $c->setExplicitFields("EXTRACT(YEAR FROM date_created)::varchar || '.' || EXTRACT(MONTH FROM date_created)::varchar as datestring, count(*) as c");
        //$c->addOrderDescending("regexp_replace(datestring, '\.[0-9]+$', '')::integer");
        //$c->addOrderDescending("regexp_replace(datestring, '^[0-9]+\.', '')::integer");
        $q = PagePeer::instance()->criteriaToQuery($c);

        $r = $db->query($q);
        $r = $r->fetchAll();
        if ($r === false) {
            $r = array();
        }
        $postCount = array();
        if ($lang == 'pl') {
            $locale = 'pl_PL';
        }
        setlocale(LC_TIME, $locale);

        foreach ($r as $mo) {
            $spl = explode('.', $mo['datestring']);
            $year = $spl[0];
            $month = $spl[1];
            $postCount[$year]['months'][$month]['count'] = $mo['c'];
            /* Month names. */
            $lang = $site->getLanguage();
            $locale = 'en_US';
            $postCount[$year]['months'][$month]['name'] = strftime('%B', mktime(6, 6, 6, $month, 6, $year));
        }

        //$c = clone($corig);
        $c->setExplicitFields("EXTRACT(YEAR FROM date_created)::varchar as datestring, count(*) as c");
        $q = PagePeer::instance()->criteriaToQuery($c);

        $r = $db->query($q);
        $r = $r->fetchAll();
        if ($r === false) {
            $r = array();
        }
        foreach ($r as $mo) {
            $postCount[$mo['datestring']]['count'] = $mo['c'];
        }

        /* Order the results. */
        /* Order years. */
        krsort($postCount, SORT_NUMERIC);

        foreach ($postCount as & $year) {
            krsort($year['months'], SORT_NUMERIC);
        }
        $uprefix = '';
        if ($attrUrlPrefix) {
            $uprefix = $attrUrlPrefix . '_';
        }

        /* Get current (selected) date (if any). */
        $date = $this->_pl->getParameterValue($uprefix."date", "GET");


        $dateA = array();
        if (preg_match('/^[0-9]{4}$/', $date)) {
            $dateA['year'] = $date;
            if (isset($postCount[$date])) {
                $postCount[$date]['selected'] = true;
            }
        }
        if (preg_match('/^[0-9]{4}\.[0-9]{1,2}$/', $date)) {
            $dateS = explode('.', $date);
            $dateA['year'] = $dateS[0];
            $dateA['month'] = $dateS[1];

            if (isset($postCount[$dateA['year']]['months'][$dateA['month']])) {
                $postCount[$dateA['year']]['months'][$dateA['month']]['selected'] = true;
            }
        }

        $runData->contextAdd('postCount', $postCount);

        $startUrlBase = '/' . $startPage;
        if ($tagString) {
            $startUrlBase .= '/'.$uprefix.'tag/'.urldecode($tagString);
        }

        $startUrlBase .= '/'.$uprefix.'date/';

        $runData->contextAdd('startUrlBase', $startUrlBase);
        //var_dump($postCount);

        $runData->contextAdd('attrUrlPrefix', $attrUrlPrefix);

        return;
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
}
