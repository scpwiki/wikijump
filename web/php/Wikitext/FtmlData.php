<?php


namespace Wikidot\Wikitext;

class FtmlPageInfo
{
    private FFI\CData $c_data;

    public function __construct(
        string $page,
        ?string $category,
        string $site,
        string $title,
        ?string $altTitle,
        array $tags,
        string $locale
    ) {
        $c_data = FtmlRaw::getInstance()->make("struct ftml_page_info");
        // TODO
    }
}
