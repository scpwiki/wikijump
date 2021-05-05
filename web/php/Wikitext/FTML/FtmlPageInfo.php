<?php
declare(strict_types = 1);

namespace Wikidot\Wikitext\FTML;

use \FFI;

/**
 * Class FtmlPageInfo, representing an input 'struct ftml_page_info' object.
 * @package Wikidot\Wikitext\FTML
 */
class FtmlPageInfo
{
    private FFI\CData $c_data;

    public function __construct(
        string $page,
        ?string $category,
        string $site,
        string $title,
        ?string $alt_title,
        array $tags,
        string $locale
    ) {
        $this->c_data = FtmlRaw::make(FtmlRaw::$FTML_PAGE_INFO);
        $this->c_data->page = $page;
        $this->c_data->category = $category;
        $this->c_data->site = $site;
        $this->c_data->title = $title;
        $this->c_data->alt_title = $alt_title;
        $tag_array = listToPointer(FtmlRaw::C_STRING, $tags);
        $this->c_data->tags_list = $tag_array->pointer;
        $this->c_data->tags_len = $tag_array->length;
        $this->c_data->locale = $locale;
    }

    public function pointer(): FFI\CData {
        return $this->c_data;
    }

    function __destruct()
    {
        FFI::free($this->c_data->tags_list);
    }
}