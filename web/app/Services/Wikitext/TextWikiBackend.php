<?php
declare(strict_types = 1);

namespace Wikijump\Services\Wikitext;

use \Wikidot\DB\Page;
use \Wikidot\DB\Site;
use \Wikidot\Utils\WikiTransformation;

class TextWikiBackend implements WikitextBackend
{
    private WikiTransformation $wt;

    public function __construct(ParseRenderMode $mode, ?PageInfo $pageInfo)
    {
        $this->wt = new WikiTransformation();

        if (!is_null($pageInfo)) {
            $site = self::getSite($pageInfo->site);
            $page = self::getPage($site->getSiteId(), $pageInfo->page);
            $this->wt->setPage($page);
        }
    }

    // Interface methods
    public function renderHtml(string $wikitext): HtmlOutput
    {
        $html = $this->wt->processSource($wikitext);
        return new HtmlOutput($html, '', [], []);
    }

    public function renderText(string $wikitext): TextOutput
    {
        throw new \Exception("Not implemented (legacy)");
    }

    public function version(): string {
        return 'Text_Wiki 0.0.1';
    }

    // Helper methods
    private static function getSite(string $siteSlug): Site
    {
        $c = new Criteria();
        $c->add('unix_name', $siteSlug);
        return SitePeer::instance()->selectOne($c);
    }

    private static function getPage(int $siteId, string $pageSlug): Page
    {
        return PagePeer::instance()->selectByName($siteId, $pageSlug);
    }
}
