<?php
declare(strict_types = 1);

namespace Wikidot\Wikitext\FTML;

use \FFI;

/**
 * Class FtmlHtmlOutput, representing a returned 'struct ftml_html_output' object.
 * @package Wikidot\Wikitext\FTML
 */
class FtmlHtmlOutput
{
    public string $html;
    public string $style;
    public array $meta;
    public array $warnings;

    public function __construct(FFI\CData $c_data) {
        $this->html = FFI::string($c_data->html);
        $this->style = FFI::string($c_data->style);
        $this->meta = FtmlHtmlMeta::fromArray($c_data->meta_list, $c_data->meta_len);
        $this->warnings = FtmlWarning::fromArray($c_data->warning_list, $c_data->warning_len);

        // Free original C data
        FtmlRaw::freeHtmlOutput($c_data);
    }
}
