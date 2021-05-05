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
        $tag_array = FtmlFfi::listToPointer(
            FtmlFfi::$C_STRING,
            $tags,
            fn(string $tag) => FtmlFfi::string($tag),
        );

        $this->c_data = FtmlFfi::make(FtmlFfi::$FTML_PAGE_INFO);
        $this->c_data->page = FtmlFfi::string($page);
        $this->c_data->category = FtmlFfi::string($category);
        $this->c_data->site = FtmlFfi::string($site);
        $this->c_data->title = FtmlFfi::string($title);
        $this->c_data->alt_title = FtmlFfi::string($alt_title);
        $this->c_data->tags_list = $tag_array->pointer;
        $this->c_data->tags_len = $tag_array->length;
        $this->c_data->locale = FtmlFfi::string($locale);
    }

    public function pointer(): FFI\CData {
        return FFI::addr($this->c_data);
    }

    function __destruct() {
        FtmlFfi::freePointer(
            $this->c_data->tags_list,
            $this->c_data->tags_len,
            fn(FFI\CData $c_data) => FFI::free($c_data),
        );

        FFI::free($this->c_data->page);
        FFI::free($this->c_data->category);
        FFI::free($this->c_data->site);
        FFI::free($this->c_data->title);
        FFI::free($this->c_data->alt_title);
        FFI::free($this->c_data->locale);
        FFI::free($this->c_data);
    }
}
