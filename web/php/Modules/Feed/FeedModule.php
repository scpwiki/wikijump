<?php

namespace Wikidot\Modules\Feed;

use Exception;
use Wikidot\Utils\CacheableModule;
use Wikidot\Utils\MagpieFeed;
use Wikidot\Utils\ProcessException;

use Wikijump\Services\Wikitext\ParseRenderMode;

use function Wikijump\Services\Wikitext\getWikitextBackend;

class FeedModule extends CacheableModule
{

    protected $timeOut = 300;

    protected $allowChangeTimeOut = true;

    private $tmpItem; // nasty parameter passing...
    private $tmpFeedArray;

    public function build($runData)
    {
        $pl = $runData->getParameterList();
        $src = $pl->getParameterValue("src", "MODULE");
        $limit =  $pl->getParameterValue("limit", "MODULE");
        $offset =  $pl->getParameterValue("offset", "MODULE");

        if ($src == null) {
            throw new ProcessException(_('No feed source specified ("src" element missing).'), "no_source");
        }

        $feedArray = array();

        if (strpos($src, ';')!==false) {
            // multiple sources!!!

            $itemArray = array();

            $urls = preg_split('/;\s*/', $src);

            foreach ($urls as $url) {
                // get the feed!!!
                try {
                    $mrss = new MagpieFeed();

                    //TODO: check if $src is a valid address?
                    $rss = $mrss->fetch($url);
                    $items = $rss->items;
                    $feedIdx = array_push($feedArray, $rss)-1;
                    for ($i=0; $i<count($items); $i++) {
                        $items[$i]['feed_idx'] = $feedIdx;
                    }
                    $itemArray = array_merge($itemArray, $items);
                } catch (Exception $e) {
                    throw new ProcessException(sprintf(_('Error processing the feed "%s". The feed cannot be accessed or contains errors. '), $url), "feed_failed");
                }
            }

            // now order by date...
            $ordertmp = array();
            foreach ($itemArray as $key => &$item) {
                // fix dates
                $timestamp = MagpieFeed::getUnixTimestamp($item);
                $ordertmp[$key] = $timestamp;
            }
            // sort.
            arsort($ordertmp, SORT_NUMERIC);
            $itemArray2 = array();
            foreach ($ordertmp as $key => $dummy) {
                $itemArray2[] = $itemArray[$key];
            }
            $itemArray = $itemArray2;
        } else {
            // get the feed!!!
            try {
                $mrss = new MagpieFeed();

                //TODO: check if $src is a valid address?
                $rss = $mrss->fetch($src);
            } catch (Exception $e) {
                throw new ProcessException(sprintf(_('Error processing the feed "%s". The feed cannot be accessed or contains errors. '), $src), "feed_failed");
            }

            $items= $rss->items;
            $feedIdx = array_push($feedArray, $rss);
            for ($i=0; $i<count($items); $i++) {
                $items[$i]['feedl_idx'] = $feedIdx;
            }
            $itemArray = $items;
        }

        $this->tmpFeedArray = $feedArray;

        $format = $pl->getParameterValue("module_body");

        if ($format == null || $format == '') {
            $format = "" .
                    "++ %%linked_title%%\n\n" .
                    "%%date%%\n\n" .
                    "%%description%%";
        }

        // process the format and create the message template
        $wt = getWikitextBackend(ParseRenderMode::FEED, null);
        $template = $wt->renderHtml($format)->html;

        // fix template
        $template = preg_replace(
            '/
            <p\s*>\s*
            (%%(
                (?:short)
                |(?:description)
                |(?:summary)
                |(?:content)
                |(?:long)
                |(?:body)
            )%%)
            \s*<\/\s*p>
            /smix',
            "<div>\\1</div>",
            $template
        );

        $fitems = array(); // formatted items
        // now for each of the feed items fill the template
        foreach ($itemArray as $item) {
            $description = $item['description'];
            if ($description === null) {
                $description = $item['summary'];
            }
            $full = $item['content']['encoded'];
            if ($full === null) {
                $full = $item['atom_content'];
            }
            if ($description === null && $full) {
                // make a shorter version????? TODO!
                $description = substr(strip_tags($full), 0, 500);
            }

            if ($full == null && $description) {
                $full = $description;
            }

            // fix dates
            $item['timestamp'] = MagpieFeed::getUnixTimestamp($item);
            if ($item['timestamp'] != '') {
                $dateString = '<span class="odate">'.$item['timestamp'].'|%e %b %Y, %H:%M %Z|agohover</span>';
            } else {
                $dateString = '';
            }
            $b = $template;
            $b = str_ireplace('%%title%%', strip_tags($item['title']), $b);
            $b = preg_replace('/%%((linked_title)|(title_linked))%%/i', preg_quote_replacement('<a href="'.$item['link'].'">'.strip_tags($item['title']).'</a>'), $b);

            // channel data
            $channel = $feedArray[$item['feed_idx']]->channel;

            $b = str_replace('%%channel_title%%', $channel['title'], $b);
            $b = preg_replace('/(%%linked_channel_title%%)|(%%channel_title_linked%%)/', preg_quote_replacement('<a href="'.$channel['link'].'">'.htmlspecialchars($channel['title']).'</a>'), $b);

            $b = str_ireplace('%%link%%', $item['link'], $b);

            $b = preg_replace('/%%((short)|(description)|(summary))%%/i', preg_quote_replacement($description), $b);

            $b = preg_replace('/%%((content)|(long)|(body))%%/i', preg_quote_replacement($full), $b);

            $b = str_ireplace('%%date%%', $dateString, $b);
            $b = preg_replace('/%%date\|(.*?)%%/i', '<span class="odate">'.$item['timestamp'].'|\\1</span>', $b);

            // start removing ads block!!!

            // custom tags
            try {
                $this->tmpItem = $item;
                $b = preg_replace_callback("/%%custom[_ ]([a-zA-Z0-9_:\/]*)%%/", array(&$this, 'processCustomTag'), $b);
                ;
            } catch (Exception $e) {
                echo $e->getMessage();
            }

            // some cleanup

            // remove ids

            $b = $this->safeString($b);
            $b = HtmlUtilities::purify($b);

            if ($channel['title']=="Slashdot") {
                // remove ads
                //$p = ';<p>\s*<a href="http://rss.slashdot.org/~a/[^"]*"><img src="http://rss.slashdot.org/~a/[^"]*" border="0" />\s*</a>\s*' .
                $p = ';<p>\s*<a href="http://rss.slashdot.org/~a/[^"]*"><img src="http://rss.slashdot.org/~a/[^"]*" border="0" />\s*</a>\s*' .
                        '</p><img src="http://rss.slashdot.org/[^"]*"\s*/>;smi';
                $b = preg_replace($p, '', $b);
            }

            $fitems[] = $b;
        }

        if (($limit !== null && is_numeric($limit)) || ($offset !== null && is_numeric($offset))) {
            if ($offset == null) {
                $offset = 0;
            }
            if ($limit !==null) {
                $fitems = array_slice($fitems, $offset, $limit);
            } else {
                $fitems = array_slice($fitems, $offset);
            }
        }

        $runData->contextAdd("format", $template);

        $runData->contextAdd("items", $fitems);
        $runData->contextAdd("src", $src);
        $runData->contextAdd("rss", $rss);
    }

    private function processCustomTag($matches)
    {

        $item = $this->tmpItem;
        $key = strtolower($matches[1]);
        if (preg_match("/^feed/", $key)) {
            $feed = $this->tmpFeedArray[$item['feed_idx']];

            // refers to the feed root
            $key = str_replace(":", "/", $key);
            ;
            list($dummy, $key1, $key2, $key3) = explode("/", $key);
            if ($key2 === null) {
                return  $feed->$key1;
            } elseif ($key3 === null) {
                $a = $feed->$key1;
                return $a[$key2];
            } else {
                $a = $feed->$key1;
                return $a[$key2][$key3];
            }
        }
        // format it!
        $key = preg_replace("/\/([a-z0-9]+:)?/i", "_", $key);

        list($key1, $key2) = explode(":", $key);

        if ($key2 == null) {
            return trim($item[$key1]);
        } else {
            return trim($item[$key1][$key2]);
        }
    }

    /**
     * Make a string safe for web publication. Removes unsafe tags, javascript etc.
     */
    private function safeString($text)
    {

        $unwantedAttributes=array(
            "on[a-z]+",
            "id",
            "xmlns"
        //  "Class"
        );

        $text = preg_replace("/<script.*?>.*?<\/script>/si", '', $text);

        // remove href="javascript:..."
        $text = preg_replace('/href\s*=\s*"javascript:[^"]*"/is', 'href="javascript:;', $text);

        // check for iframe and javascript src
        $ld = preg_quote($_SERVER["HTTP_HOST"]);
        $text = preg_replace('/<iframe.*?src="https?:\/\/'.$ld.'[^"]*"[^>]*?((\/>)|(>\s*<\s*\/iframe>))/si', '', $text);
        //remove unwanter attributes
        $u2 = array();
        foreach ($unwantedAttributes as $ua) {
            $u2[] = '/(<[^>]*?)'.$ua.'="[^"]*"([^>]*>)/si';
        }
        do {
            $text2 = $text;
            $text = preg_replace($u2, "\\1 \\2", $text);
        } while ($text2 != $text);

        return $text ;
    }
}
