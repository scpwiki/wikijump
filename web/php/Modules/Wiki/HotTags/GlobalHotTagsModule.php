<?php

namespace Wikidot\Modules\Wiki\HotTags;


use Illuminate\Support\Facades\Cache;
use Ozone\Framework\Database\Criteria;
use Ozone\Framework\SmartyModule;
use Wikidot\DB\PagePeer;
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

        $struct = Cache::get($key);

        $cacheTimestamp = $struct['timestamp'];
        $changeTimestamp = Cache::get($tkey);

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

        Cache::put($key, $struct, 1000);

        if (!$changeTimestamp) {
            $changeTimestamp = $now;
            Cache::put($tkey, $changeTimestamp, 3600);
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
                    $tag_counts[$tag] = 1;
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
        $tags = [];
        foreach ($tag_counts as $tag => $weight) {
            $a = $weightRange === 0 ? 0 : ($weight - $minWeight) / $weightRange;
            $fontSize = round($sizeSmall + ($sizeBig - $sizeSmall) * $a);

            // hadle colors... woooo! excited!

            $color = [
                'r' => round($colorSmall[0] + ($colorBig[0] - $colorSmall[0]) * $a),
                'g' => round($colorSmall[1] + ($colorBig[1] - $colorSmall[1]) * $a),
                'b' => round($colorSmall[2] + ($colorBig[2] - $colorSmall[2]) * $a),
            ];

            $tags[$tag] = [
                'size' => $fontSize . $fsformat,
                'color' => $color,
            ];
        }

        $runData->contextAdd('tags', $tags);
        $runData->contextAdd('href', $target);
    }
}
