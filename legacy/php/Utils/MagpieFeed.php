<?php

namespace Wikidot\Utils;

use Illuminate\Support\Facades\Cache;
use Ozone\Framework\Ozone;

define("MAGPIE_DIR", WIKIJUMP_ROOT."/lib/magpierss/");
define("MAGPIE_CACHE_ON", false);
define("MAGPIE_OUTPUT_ENCODING", "UTF-8");

require(WIKIJUMP_ROOT."/lib/magpierss/rss_fetch.inc");

class MagpieFeed
{

    public function fetch($url)
    {

        // check if not already in cache (memcache only)

        $key = "feed..".$url;
        $o = Cache::get($key);
        if ($o != false && $o != null) {
            return $o;
        }

        // not in cache, proceed!!!

        $o = fetch_rss($url);
        Cache::put($key, $o, 300);

        return $o;
    }

    public static function getUnixTimestamp($item)
    {
        $rss_2_date = $item['pubdate'];
        $rss_1_date = $item['dc']['date'];
        $atom_date  = $item['issued'];

        if ($atom_date != "") {
            $date = parse_w3cdtf($atom_date);
        }
        if ($rss_1_date != "") {
            $date = parse_w3cdtf($rss_1_date);
        }
        if ($rss_2_date != "") {
            $date = strtotime($rss_2_date);
        }

        return $date;
    }
}
