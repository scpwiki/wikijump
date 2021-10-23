<?php
declare(strict_types=1);

namespace Wikijump\Services\Wikitext;

use Wikijump\Services\Wikitext\FFI\FtmlFfi;

/**
 * Class FtmlInterface, implements a compatible interface for working with FTML.
 * @package Wikijump\Services\Wikitext
 */
class FtmlBackend extends WikitextBackend
{
    private PageInfo $page_info;
    private WikitextSettings $settings;

    public function __construct(int $mode, ?PageInfo $page_info)
    {
        $this->page_info = $page_info ?? self::defaultPageInfo();
        $this->settings = WikitextSettings::fromMode($mode);
    }

    // Interface methods
    public function renderHtml(string $wikitext): HtmlOutput
    {
        return FtmlFfi::renderHtml($wikitext, $this->page_info, $this->settings);
    }

    public function renderText(string $wikitext): TextOutput
    {
        return FtmlFfi::renderText($wikitext, $this->page_info, $this->settings);
    }

    public function version(): string
    {
        return FtmlFfi::version();
    }

    private static function defaultPageInfo(): PageInfo
    {
        return new PageInfo(
            '_anonymous',
            null,
            'test',
            '_anonymous',
            null,
            [],
            'default',
        );
    }
}
