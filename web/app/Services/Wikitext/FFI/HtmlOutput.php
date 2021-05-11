<?php
declare(strict_types = 1);

namespace Wikijump\Services\Wikitext\FFI;

use \FFI;

/**
 * Class HtmlOutput, representing a returned 'struct ftml_html_output' object.
 * @package Wikijump\Services\Wikitext\FFI
 */
class HtmlOutput
{
    public string $body;
    public array $styles;
    public array $meta;
    public array $warnings;

    public function __construct(FFI\CData $c_data) {
        $this->body = FFI::string($c_data->body);
        $this->styles = self::stylesFromArray($c_data->styles_list, $c_data->styles_len);
        $this->meta = HtmlMeta::fromArray($c_data->meta_list, $c_data->meta_len);
        $this->warnings = ParseWarning::fromArray($c_data->warning_list, $c_data->warning_len);

        // Free original C data
        FtmlFfi::freeHtmlOutput($c_data);
        FFI::free($c_data);
    }

    public static function stylesFromArray(FFI\CData $pointer, int $length): array {
        return FtmlFfi::pointerToList(
            $pointer,
            $length,
            fn(FFI\CData $c_data) => FFI::string($c_data),
        );
    }
}
