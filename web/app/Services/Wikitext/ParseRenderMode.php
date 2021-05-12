<?php
declare(strict_types = 1);

namespace Wikijump\Services\Wikitext;

use \Wikijump\Common\Enum;

/**
 * Enum ParseRenderMode, for representing the context in which parsing and rendering is being carried out.
 *
 * PAGE              -- Compiling a regular page
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
    const FORUM_POST = 1;
    const DIRECT_MESSAGE = 2;
    const FEED = 3;
    const LIST = 4;
    CONST TABLE_OF_CONTENTS = 5;
}
