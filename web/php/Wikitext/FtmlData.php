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
        ?string $alt_title,
        array $tags,
        string $locale
    ) {
        $this->c_data = FtmlRaw::getInstance()->make("struct ftml_page_info");
        $this->c_data->page = $page;
        $this->c_data->category = $category;
        $this->c_data->site = $site;
        $this->c_data->title = $title;
        $this->c_data->alt_title = $alt_title;
        $tag_array = listToPointer($tags);
        $this->c_data->tags_list = $tag_array->pointer;
        $this->c_data->tags_len = $tag_array->length;
        $this->c_data->locale = $locale;
    }

    function __destruct() {
        parent::__destruct();
        FFI::free($this->c_data->tags_list);
    }
}

class FtmlHtmlOutput
{
    private FFI\CData $c_data;

    public function __construct(FFI\CData $c_data) {
        $this->c_data = $c_data;
    }

    function __destruct() {
        parent::__destruct();
        FtmlRaw::getInstance()->freeHtmlOutput($this->c_data);
    }
}

class FtmlTextOutput
{
    private FFI\CData $c_data;

    public function __construct(FFI\CData $c_data) {
        $this->c_data = $c_data;
    }

    function __destruct() {
        parent::__destruct();
        FtmlRaw::getInstance()->freeTextOutput($this->c_data);
    }
}

/**
 * Converts a list in the form of a PHP array into a pointer
 * suitable for passing into C FFIs.
 *
 * All of the objects in the array must already be ready for passing.
 *
 * @returns array with keys "pointer" and "length"
 */
function listToPointer(array $list): array {
    // Allocate heap array
    $length = count($list);
    $pointer = FtmlRaw::getInstance()->make("char *[$length]");

    // Copy string elements
    foreach ($list as $index => $item) {
        $pointer[$index] = $item;
    }

    return [
        'pointer' => $pointer,
        'length' => $length,
    ];
}
