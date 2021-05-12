<?php

namespace Wikidot\Utils;

use Ozone\Framework\Database\Criteria;
use Ozone\Framework\Database\Database;
use Ozone\Framework\Ozone;
use Text_Wiki;
use Wikidot\DB\PageTagPeer;

class WikiTransformation
{
    /**
     * The array stores all internal links within a page.
     * Each element is an array (page_id, page_unix_name)
     */
    public static $internalLinksExist;
    public static $externalLinks;
    public static $internalLinksNotExist;

    private $page;

    public $wiki;

    public function __construct($initialize = true)
    {
        if ($initialize) {
            self::$internalLinksExist = array();
            self::$internalLinksNotExist = array();
            self::$externalLinks = array();

            // initialize Wiki engine with default values
            $wiki = new Text_Wiki();
            $viewUrl = '/%s';
            $wiki->setRenderConf('freelink', 'view_url', $viewUrl);
            $wiki->setRenderConf('freelink', 'new_url', $viewUrl);
            $wiki->setRenderConf('url', 'images', false);

            $wiki->setRenderConf('freelink', 'new_text', '');
            $wiki->setRenderConf('freelink', 'css_new', 'newpage');
            $wiki->setRenderConf('table', 'css_table', 'wiki-content-table');

            $wiki->setRenderConf('freelink', 'exists_callback', __NAMESPACE__ . '\wikiPageExists');
            $wiki->setRenderConf('wikilink', 'exists_callback', __NAMESPACE__ . '\wikiPageExists');

            $interWikis = array(
                'wikipedia' => 'https://en.wikipedia.org/wiki/%s',
                'wikipedia.pl' => 'https://pl.wikipedia.org/wiki/%s',
                'pl.wikipedia' => 'https://pl.wikipedia.org/wiki/%s',
                'google' => 'https://www.google.com/search?q=%s',
                'dictionary' => 'https://dictionary.reference.com/browse/%s'
            );

            // configure the interwiki rule
            $wiki->setRenderConf('interwiki', 'sites', $interWikis);
            $wiki->setParseConf('interwiki', 'sites', $interWikis);

            $wiki->disableRule('wikilink');
            $this->wiki = $wiki;
        }
    }

    public function processSource($source)
    {

        $wiki = $this->wiki;
        if (preg_match('/_template$/', $wiki->vars['pageName'])) {
            $wiki->disablerule('separator');
        }
        $out = $wiki->transform($source);
        return HtmlUtilities::purify($out);
    }

    public function setPage($page)
    {
        $pageSlug = $page->getUnixName();
        $this->page = $page;
        $this->wiki->vars['page'] = $page;
        $this->wiki->vars['pageTitle'] = $page->getTitleOrUnixName();
        $this->wiki->vars['pageName'] = $pageSlug;
        $this->wiki->setRenderConf('image', 'base', '/local--files/'.$pageSlug.'/');
        $this->wiki->setRenderConf('file', 'base', '/local--files/'.$pageSlug.'/');
    }

    public function setMode($mode)
    {
        $wiki = $this->wiki;
        switch ($mode) {
            case 'default':
                break;
            case 'pm':
            case 'post':
                // disable a few rules
                $wiki->disableRule("include");
                $wiki->disableRule("modulepre");
                $wiki->disableRule("module");
                $wiki->disableRule("module654");
                $wiki->disableRule("toc");
                $wiki->disableRule("Social");
                $wiki->disableRule("button");

                //configure
                $wiki->setRenderConf($this->transformationFormat, 'heading', 'use_id', false);
                $wiki->setRenderConf($this->transformationFormat, 'footnote', 'id_prefix', rand(0, 1000000).'-');
                $wiki->setRenderConf($this->transformationFormat, 'bibitem', 'id_prefix', rand(0, 1000000).'-');
                $wiki->setRenderConf($this->transformationFormat, 'math', 'id_prefix', rand(0, 1000000).'-');
                $wiki->setRenderConf($this->transformationFormat, 'file', 'no_local', true);
                $wiki->setRenderConf($this->transformationFormat, 'image', 'no_local', true);
                $wiki->setRenderConf($this->transformationFormat, 'gallery', 'no_local', true);
                break;
            case 'list':
                $wiki->setRenderConf($this->transformationFormat, 'heading', 'use_id', false);
                $wiki->setRenderConf($this->transformationFormat, 'footnote', 'id_prefix', rand(0, 1000000).'-');
                $wiki->setRenderConf($this->transformationFormat, 'bibitem', 'id_prefix', rand(0, 1000000).'-');
                $wiki->setRenderConf($this->transformationFormat, 'math', 'id_prefix', rand(0, 1000000).'-');
                break;
            case 'feed':
                // disable a few rules
                $wiki->disableRule("include");
                $wiki->disableRule("modulepre");
                $wiki->disableRule("module");
                $wiki->disableRule("module654");
                $wiki->disableRule("toc");
                $wiki->disableRule("footnote");
                $wiki->disableRule("math");
                $wiki->disableRule("equationreference");
                $wiki->disableRule("Footnoteitem");
                $wiki->disableRule("Footnoteblock");
                $wiki->disableRule("Bibitem");
                $wiki->disableRule("Bibliography");
                $wiki->disableRule("Bibcite");
                $wiki->disableRule("Gallery");
                $wiki->disableRule("File");
                $wiki->disableRule("Social");

                // configure
                $wiki->setRenderConf('heading', 'use_id', false);
                $wiki->setRenderConf('file', 'no_local', true);
                $wiki->setRenderConf('image', 'no_local', true);
                $wiki->setRenderConf('image', 'post_vars', true);
                $wiki->setParseConf('url', 'post_vars', true);
                break;
            default:
                throw Exception("Invalid Wiki engine mode.");
        }
    }
}

// Used by setRenderConf() via dynamic dispatch, line 50 above.

function wikiPageExists($pageName)
{
    if ($GLOBALS['site'] == null) {
        $runData = Ozone::getRunData();
        $siteId = $runData->getTemp("site")->getSiteId();
    } else {
        $siteId = $GLOBALS['site']->getSiteId();
    }
    $q = "SELECT page_id FROM page WHERE unix_name='".db_escape_string($pageName)."' AND site_id='".db_escape_string($siteId)."' LIMIT 1";
    $db = Database::connection();
    $r = $db->query($q);
    if ($row = $r->nextRow()) {
        return $row['page_id'];
    } else {
        return false;
    }
}
