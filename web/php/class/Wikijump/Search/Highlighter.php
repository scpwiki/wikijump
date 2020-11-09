<?php

namespace Wikijump\Search;

use Zend_Search_Lucene_Search_QueryParser;
use DOMDocument;
use DOMXPath;

class Highlighter
{

    public static function highlightIfSuitable($html, $request_uri, $referer)
    {

        if (self::suitable($request_uri) && $query = self::query($referer)) {
            $queryObj = Zend_Search_Lucene_Search_QueryParser::parse($query);
            $out = $queryObj->highlightMatches($html);

            if (! $out) {
                return $html;
            }

            $htmlNice = self::joinHtml($html, $out);

            if ($htmlNice) {
                return $htmlNice;
            }
        }

        return $html;
    }

    protected static function query($referer)
    {

        $host = parse_url($referer, PHP_URL_HOST);
        $path = parse_url($referer, PHP_URL_PATH);
        $query = parse_url($referer, PHP_URL_QUERY);

        $a = array();
        parse_str($query, $a);

        // Google search
        if ($path == '/search' && isset($a['q'])) {
            return $a['q'];
        }

        // Yahoo search
        if (preg_match('|^/search;|', $path) && isset($a['p'])) {
            return $a['p'];
        }

        // Wikijump search
        $a = array();
        if (preg_match(";/search:(site|all)/(a/[pf]*/)?q/([^/]*)($|/);", $path, $a)) {
            return $a[3];
        }

        return null;
    }

    // highlight is not suitable for the main page (/) and search pages themselves
    protected static function suitable($request_uri)
    {

        return ! preg_match(";^/($|search:);", $request_uri);
    }

    protected static function joinHtml($html, $out)
    {
        $dom = new DOMDocument();
        @$dom->loadHTML($out);

        $x = new DOMXPath($dom);
        $xa = $x->query('id("main-content")');
        $out_main = $xa->item(0);

        if (! $out_main) {
            return null;
        }

        $dom = new DOMDocument();
        @$dom->loadHTML($html);

        $x = new DOMXPath($dom);
        $xa = $x->query('//div[@id="main-content"]');
        $main = $xa->item(0);

        if (! $main) {
            return null;
        }

        $x = new DOMXPath($dom);
        $xa = $x->query('//div[@id="content-wrap"]');
        $wrapper = $xa->item(0);

        if (! $wrapper) {
            return null;
        }

        $out_main = $dom->importNode($out_main, true);
        $wrapper->replaceChild($out_main, $main);

        return $dom->saveHTML();
    }
}
