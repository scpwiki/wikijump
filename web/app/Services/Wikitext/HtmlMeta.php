<?php
declare(strict_types = 1);

namespace Wikijump\Services\Wikitext;

use \FFI;

/**
 * Class HtmlMeta, representing a returned 'struct ftml_html_meta' object.
 * @package Wikijump\Services\Wikitext
 */
class HtmlMeta
{
    public string $tagType;
    public string $name;
    public string $value;

    public function __construct(FFI\CData $c_data) {
        $this->tagType = self::getTagType($c_data->tag_type);
        $this->name = FFI::string($c_data->name);
        $this->value = FFI::string($c_data->value);
    }

    public static function fromArray(FFI\CData $pointer, int $length): array {
        return FtmlFfi::pointerToList(
            $pointer,
            $length,
            fn(FFI\CData $c_data) => new HtmlMeta($c_data),
        );
    }

    private static function getTagType(int $c_tag): string {
        switch ($c_tag) {
            case FtmlFfi::$META_NAME:
                return 'name';
            case FtmlFfi::$META_HTTP_EQUIV:
                return 'http-equiv';
            case FtmlFfi::$META_PROPERTY:
                return 'property';
            default:
                throw new Error("Invalid HTML meta tag type C enum value: $c_tag");
        }
    }
}
