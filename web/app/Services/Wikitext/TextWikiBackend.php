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

        // Set page data
        if (!is_null($pageInfo)) {
            $site = self::getSite($pageInfo->site);
            $page = self::getPage($site->getSiteId(), $pageInfo->page);
            $this->wt->setPage($page);
        }

        // Set parse mode
        switch ($mode) {
            case ParseRenderMode::PAGE:
            case ParseRenderMode::TABLE_OF_CONTENTS:
                $this->wt->setMode('default');
                break;
            case ParseRenderMode::FORUM_POST:
                $this->wt->setMode('post');
                break;
            case ParseRenderMode::DIRECT_MESSAGE:
                $this->wt->setMode('pm');
                break;
            case ParseRenderMode::FEED:
                $this->wt->setMode('feed');
                break;
            case ParseRenderMode::LIST:
                $this->wt->setMode('list');
                break;
            default:
                throw new Exception("Unknown parse/render mode: $mode");
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
