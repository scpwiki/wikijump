<?php

namespace Wikidot\Modules\Wiki\HotTags;


use Ozone\Framework\Database\Database;
use Ozone\Framework\Ozone;
use Ozone\Framework\SmartyModule;
use Wikidot\Utils\ProcessException;

class GlobalHotTagsModule extends SmartyModule
{

    public function render($runData)
    {

        $site = $runData->getTemp("site");
        $pl = $runData->getParameterList();

        $parmArray = $pl->asArray();
        unset($parmArray['tag']);
        $parmHash = md5(serialize($parmArray));

        $key = 'global_hot_tags_v..'.$site->getSiteId().'..'.$parmHash;
        $tkey = 'global_hot_tags_lc..'.$site->getSiteId(); // last change timestamp

        $mc = OZONE::$memcache;
        $struct = $mc->get($key);

        $cacheTimestamp = $struct['timestamp'];
        $changeTimestamp = $mc->get($tkey);

        if ($struct) {
            // check the times

            if ($changeTimestamp && $changeTimestamp <= $cacheTimestamp) {
                $out = $struct['content'];
                return $out;
            }
        }

        $out = parent::render($runData);

        // and store the data now
        $struct = array();
        $now = time();
        $struct['timestamp'] = $now;
        $struct['content'] = $out;

        $mc->set($key, $struct, 0, 1000);

        if (!$changeTimestamp) {
            $changeTimestamp = $now;
            $mc->set($tkey, $changeTimestamp, 0, 3600);
        }

        return $out;
    }

    public function build($runData)
    {

        $pl = $runData->getParameterList();

        // get some cool parameters

        $maxFontSize = $pl->getParameterValue("maxFontSize");
        $minFontSize = $pl->getParameterValue("minFontSize");

        $minColor = $pl->getParameterValue("minColor");
        $maxColor = $pl->getParameterValue("maxColor");

        $target = $pl->getParameterValue("target");

        $limit = $pl->getParameterValue("limit");

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

        if ($limit && is_numeric($limit) && $limit<50) {
        } else {
            $limit = 50;
        }

        $site = $runData->getTemp("site");

        $db = Database::connection();
        //select tags
        if ($category == null) {
            $q = "SELECT * FROM (SELECT tag, COUNT(*) AS weight FROM page_tag  WHERE site_id='".$site->getSiteId()."' GROUP BY tag ORDER BY weight DESC LIMIT $limit) AS foo ORDER BY tag";
        } else {
            $q = "SELECT * FROM (SELECT tag, COUNT(*) AS weight FROM page_tag, page  WHERE page_tag.site_id='".$site->getSiteId()."' " .
                    " AND page.category_id='".$category->getCategoryId()."' " .
                    " AND page.page_id = page_tag.page_id " .
                    "GROUP BY tag ORDER BY weight DESC LIMIT $limit) AS foo ORDER BY tag";

            $runData->contextAdd("category", $category);
        }

        $res = $db->query($q);
        $tags = $res->fetchAll();

        $minWeight = 10000000;
        $maxWeight = 0;

        if (!$tags) {
            return;
        }
        foreach ($tags as $tag) {
            if ($tag['weight'] > $maxWeight) {
                $maxWeight = $tag['weight'];
            }
            if ($tag['weight'] < $minWeight) {
                $minWeight = $tag['weight'];
            }
        }

        $weightRange = $maxWeight - $minWeight;

        // now set color and font size for each of the tags.

        foreach ($tags as &$tag) {
            if ($weightRange == 0) {
                $a = 0;
            } else {
                $a = ($tag['weight']-$minWeight)/$weightRange;
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
}
