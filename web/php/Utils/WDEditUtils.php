<?php

namespace Wikidot\Utils;

use Wikijump\Services\Wikitext\ParseRenderMode;

use function Wikijump\Services\Wikitext\WikitextBackend;

class WDEditUtils
{

    /**
     * Checks if a page is editable by sections.
     */
    public static function sectionsEditable($content)
    {

        // create a xml tree? not always valid xhtml.
        // rather check if <h[1-6] id="toc.+*? > elements are inside any div
        // the test should be already in the javascript but we should not rely on it...

        // first count all occurences of <h[1-6]> tags.
        $content = preg_replace("/%+/", '', $content);
        $content = preg_replace('/<(h[1-6]) id="toc.+?>.+?<\/\\1>/s', "%%%%", $content);
        $count1 = preg_match_all("/%%%%/", $content, $matches);
        // now remove all tags with contents and recount.

        // now remove all tags with insides
        $content = preg_replace("/<(\w+)[^>]*?>.*?<\/\\1>/sm", "", $content);
        $count2 = preg_match_all("/%%%%/", $content, $matches2);
        if ($count2 == 0) {
            return false;
        }

        if ($count1 == $count2) {
            return true;
        } else {
            return false;
        }
    }

    /**
     * Creates a section mapping between the source code and the compiled code.
     * Using this mapping one can easily find source lines corresponding to
     * particular section toc[0-9]+ in the compiled content.
     */
    public static function sectionMapping($source)
    {
        // alter all of the headings in the source
        $s1 = explode("\n", $source);
        for ($i = 0; $i<count($s1); $i++) {
            $s1[$i] = preg_replace('/^(\+{1,6}) (.*)/m', '${1} stoc'.$i, $s1[$i]);
        }
        $source = implode("\n", $s1);
        $totalLines = count($s1);

        $wt = WikitextBackend::make(ParseRenderMode::TABLE_OF_CONTENTS, null);
        $compiled = $wt->renderHtml($source)->body;

        // now find all the occurences of headings in the compiled content.
        $pattern = '/<h([1-6]) id="toc([0-9]+)"[^>]*?>(?:\s*<span>)?\s*stoc([0-9]+)/';
        preg_match_all($pattern, $compiled, $matches, PREG_SET_ORDER);
        // array of the form:
        //      key: sequential
        //      value: array(toc_id,level,start, end) - start&end lines of the section
        $map0 = array();
        foreach ($matches as $m) {
            $map0[] = array('id' => $m[2], 'level' => $m[1], 'start' => $m[3]);
        }

        // now fix it and include ends (& levels of course).
        $map = array();

        for ($key = 0; $key<count($map0); $key++) {
            $m = $map0[$key];
            $mlevel = $m['level'];
            // look for an end

            $endI = null;
            for ($i=$key+1; $i<count($map0); $i++) {
                if ($map0[$i]['level']<=$mlevel) {
                    $endI = $map0[$i]['start']-1;
                    break;
                }
            }
            if ($endI == null) {
                $endI = $totalLines-1;
            }
            $mm = array();
            $mm['level'] = $m['level'];
            $mm['start'] = $m['start'];
            $mm['end'] = $endI;

            $map[$m['id']] = $mm;
        }
        return $map;
    }
}
