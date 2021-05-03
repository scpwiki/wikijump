<?php
declare(strict_types = 1);

namespace Wikidot\Wikitext\FTML;

class FtmlHtmlMeta
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
        return pointerToList(
            $pointer,
            $length,
            fn(FFI\CData $c_data) => new FtmlHtmlMeta($c_data),
        );
    }

    private static function getTagType(FFI\CData $c_tag): string {
        switch ($c_tag) {
            case FtmlRaw::META_NAME:
                return 'name';
            case FtmlRaw::META_HTTP_EQUIV:
                return 'http-equiv';
            case FtmlRaw::META_PROPERTY:
                return 'property';
            default:
                throw new Error("Invalid HTML meta tag type C enum value: $c_tag");
        }
    }
}