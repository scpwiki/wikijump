<?php
declare(strict_types=1);

namespace Wikijump\Services\Wikitext;

use Exception;
use Wikijump\Common\Enum;

/**
 * Enum ParseRenderMode, for representing the context in which parsing and rendering is being carried out.
 *
 * PAGE              -- Compiling a regular page
 * DRAFT             -- Compiling a draft for a page
 * FORUM_POST        -- Compiling a forum post
 * DIRECT_MESSAGE    -- Compiling a direct message
 * FEED              -- RSS feeds
 * LIST              -- ListPages module and other dynamic lists
 * TABLE_OF_CONTENTS -- Producing a table of contents source mapping
 *
 * @package Wikijump\Services\Wikitext
 */
final class ParseRenderMode extends Enum
{
    const PAGE = 0;
    const DRAFT = 1;
    const FORUM_POST = 2;
    const DIRECT_MESSAGE = 3;
    const FEED = 4;
    const LIST = 5;
    const TABLE_OF_CONTENTS = 6;

    public static function toNativeMode(int $c_mode): int
    {
        switch ($c_mode) {
            case FtmlFfi::WIKITEXT_MODE_PAGE:
                return self::PAGE;
            case FtmlFfi::WIKITEXT_MODE_DRAFT:
                return self::DRAFT;
            case FtmlFfi::WIKITEXT_MODE_FORUM_POST:
                return self::FORUM_POST;
            case FtmlFfi::WIKITEXT_MODE_DIRECT_MESSAGE:
                return self::DIRECT_MESSAGE;
            case FtmlFfi::WIKITEXT_MODE_LIST:
                return self::LIST;
            default:
                throw new Exception("No corresponding enum mode for wikitext enum value $c_mode");
        }
    }

    public static function toFfiMode(int $parse_render_mode): int
    {
        switch ($parse_render_mode) {
            case self::PAGE:
                return FtmlFfi::WIKITEXT_MODE_PAGE;
            case self::DRAFT:
                return FtmlFfi::WIKITEXT_MODE_DRAFT;
            case self::FORUM_POST:
                return FtmlFfi::WIKITEXT_MODE_FORUM_POST;
            case self::DIRECT_MESSAGE:
                return FtmlFfi::WIKITEXT_MODE_DIRECT_MESSAGE;
            case self::LIST:
                return FtmlFfi::WIKITEXT_MODE_LIST;
            case self::FEED:
            case self::TABLE_OF_CONTENTS:
            default:
                throw new Exception("No corresponding wikitext mode for enum value $parse_render_mode");
        }
    }
}
