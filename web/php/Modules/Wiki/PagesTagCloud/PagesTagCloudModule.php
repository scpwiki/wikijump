<?php

namespace Wikidot\Modules\Wiki\PagesTagCloud;


use Illuminate\Support\Facades\Cache;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\SmartyModule;
use Wikidot\DB\CategoryPeer;
use Wikidot\DB\PagePeer;
use Wikidot\Utils\ProcessException;

class PagesTagCloudModule extends SmartyModule
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

        /*
         * Read all parameters.
         */

        $categoryName = $this->_readParameter(array('category', 'categories'), false);

        $categoryName = strtolower($categoryName);

        $parmArray = $pl->asArray();
        unset($parmArray['tag']);
        unset($parmArray['wiki_page']);
        $parmHash = md5(serialize($parmArray));
        $this->parameterhash = $parmHash;

        $valid = true;

        if ($categoryName == '__current__') {
            /* Use the current category! */
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

        /* Default to ALL. */
        if (!$categoryName) {
            $categoryName = '*';
        }

        $key = 'pagetagcloud_v..' . $site->getUnixName() . '..' . $categoryName . '..' . $parmHash;

        $struct =Cache::get($key);
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

//  public function render($runData){
//      $site = $runData->getTemp("site");
//      $pl = $runData->getParameterList();
//
//      $parmArray = $pl->asArray();
//      unset($parmArray['tag']);
//      $parmHash = md5(serialize($parmArray));
//
//      $key = 'page_tags_v..'.$site->getSiteId().'..'.$parmHash;
//      $tkey = 'page_tags_lc..'.$site->getSiteId(); // last change timestamp
//
//      $mc = OZONE::$memcache;
//      $struct = $mc->get($key);
//
//      $cacheTimestamp = $struct['timestamp'];
//      $changeTimestamp = $mc->get($tkey);
//
//      if($struct){
//          // check the times
//
//          if($changeTimestamp && $changeTimestamp <= $cacheTimestamp){
//
//              $out = $struct['content'];
//              return $out;
//          }
//      }
//
//      $out = parent::render($runData);
//
//      // and store the data now
//      $struct = array();
//      $now = time();
//      $struct['timestamp'] = $now;
//      $struct['content'] = $out;
//
//      $mc->set($key, $struct, 0, 1000);
//
//      if(!$changeTimestamp){
//          $changeTimestamp = $now;
//          $mc->set($tkey, $changeTimestamp, 0, 3600);
//      }
//
//      return $out;
//  }

    public function build($runData)
    {

        $pl = $runData->getParameterList();

        // get some cool parameters

        $maxFontSize = $pl->getParameterValue("maxFontSize", "MODULE");
        $minFontSize = $pl->getParameterValue("minFontSize", "MODULE");

        $minColor = $pl->getParameterValue("minColor", "MODULE");
        $maxColor = $pl->getParameterValue("maxColor", "MODULE");

        $target = $pl->getParameterValue("target", "MODULE");

        $categoryName =  $pl->getParameterValue("category", "MODULE");

        if (!$target) {
            $target = "/system:page-tags/tag/";
        } else {
            $target = preg_replace('/^\/?/', '/', $target);
            $target = preg_replace('/\/?$/', '/', $target);
            $target .= 'tag/';
        }

        // check for font sizes
        if ($maxFontSize && $minFontSize) {
            preg_match('/^([0-9]+)(%|em|px)$/', $maxFontSize, $matches);

            $fsformat = $matches[2];
            if ($fsformat == null) {
                throw new ProcessException(_("Unsupported format for font size. Use px, em or %."));
            }
            $sizeBig = $matches[1];

            preg_match('/^([0-9]+)(%|em|px)$/', $minFontSize, $matches);
            if ($fsformat != $matches[2]) {
                throw new ProcessException(_("Format for minFontSize and maxFontSize must be the same (px, em or %)."));
            }
            $sizeSmall = $matches[1];
        } else {
            $sizeSmall = 100; // percent
            $sizeBig = 300; // percent
            $fsformat = "%";
        }

        // get colors
        if ($maxColor && $minColor) {
            if (!preg_match('/^[0-9]+,[0-9]+,[0-9]+$/', $maxColor)
                    || !preg_match('/^[0-9]+,[0-9]+,[0-9]+$/', $minColor)) {
                throw new ProcessException(_('Unsupported color format. ' .
                        'Use "RRR,GGG,BBB" for Red,Green,Blue each within 0-255 range.'));
            }
            $colorSmall = explode(',', $minColor);
            $colorBig = explode(',', $maxColor);
        } else {
            $colorSmall = array(128,128,192);
            $colorBig = array(64,64,128);
        }

        $site = $runData->getTemp("site");

        if ($categoryName) {
            $category = CategoryPeer::instance()->selectByName($categoryName, $site->getSiteId());
            if ($category == null) {
                throw new ProcessException(sprintf(_('Category "%s" cannot be found.'), $categoryName));
            }
        }

        // Fetch tags and their counts
        $c = new Criteria();
        $c->add('site_id', $site->getSiteId());
        $pages = PagePeer::instance()->select($c);
        $tag_counts = [];

        foreach ($pages as $page) {
            foreach ($page->getTagsArray() as $tag) {
                if (isset($tag_counts[$tag])) {
                    $tag_counts[$tag]++;
                } else {
                    $tag_counts[$tag] = 0;
                }
            }
        }

        // Build weights (legacy Wikidot)
        $minWeight = 10000000;
        $maxWeight = 0;

        foreach ($tag_counts as $tag => $weight) {
            if ($weight > $maxWeight) {
                $maxWeight = $weight;
            }
            if ($weight < $minWeight) {
                $minWeight = $weight;
            }
        }

        $weightRange = $maxWeight - $minWeight;

        // now set color and font size for each of the tags.

        foreach ($tag_counts as $tag => $weight) {
            if ($weightRange == 0) {
                $a = 0;
            } else {
                $a = ($weight - $minWeight) / $weightRange;
            }

            $fontSize = round($sizeSmall + ($sizeBig-$sizeSmall)*$a);

            // hadle colors... woooo! excited!

            $color = array();
            $color['r'] = round($colorSmall[0] + ($colorBig[0] - $colorSmall[0])*$a);
            $color['g'] = round($colorSmall[1] + ($colorBig[1] - $colorSmall[1])*$a);
            $color['b'] = round($colorSmall[2] + ($colorBig[2] - $colorSmall[2])*$a);

            $tag['size'] = $fontSize.$fsformat;
            $tag['color'] = $color;
        }

        $runData->contextAdd("tags", $tags);
        $runData->contextAdd("href", $target);
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
