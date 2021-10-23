<?php
declare(strict_types=1);

namespace Wikijump\Services\Wikitext;

use Ozone\Framework\Database\Criteria;
use Wikidot\DB\Page;
use Wikidot\DB\PagePeer;
use Wikidot\DB\Site;
use Wikidot\DB\SitePeer;
use Wikidot\Utils\WikiTransformation;

class TextWikiBackend extends WikitextBackend
{
    private WikiTransformation $wt;

    public function __construct(int $mode, ?PageInfo $page_info)
    {
        $this->wt = new WikiTransformation();

        // Set page data
        if ($page_info !== null) {
            $site = self::getSite($page_info->site);
            $page = self::getPage($site->getSiteId(), $page_info->page);
            $this->wt->setPage($page);
        }

        // Set parse mode
        switch ($mode) {
            case ParseRenderMode::PAGE:
            case ParseRenderMode::DRAFT:
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
        $link_stats = Backlinks::fromWikiObject($this->wt->wiki);
        return new HtmlOutput($html, [], [], [], $link_stats);
    }

    public function renderText(string $wikitext): TextOutput
    {
        $html = $this->wt->processSource($wikitext);
        $text = strip_tags($html);
        return new TextOutput($text, []);
    }

    public function version(): string
    {
        return 'Text_Wiki 0.0.1';
    }

    // Helper methods
    private static function getSite(string $site_slug): Site
    {
        $c = new Criteria();
        $c->add('unix_name', $site_slug);
        return SitePeer::instance()->selectOne($c);
    }

    private static function getPage(string $site_id, string $page_slug): Page
    {
        return PagePeer::instance()->selectByName($site_id, $page_slug);
    }
}
