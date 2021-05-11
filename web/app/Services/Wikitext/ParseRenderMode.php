<?php
declare(strict_types = 1);

namespace Wikijump\Services\Wikitext;

use \Wikijump\Common\Enum;

/**
 * Enum ParseRenderMode, for representing the context in which parsing and rendering is being carried out.
 *
 * FEED           -- RSS feeds
 * LIST           -- ListPages module and other dynamic lists
 * PAGE           -- Compiling a regular page
 * FORUM_POST     -- Compiling a forum post
 * DIRECT_MESSAGE -- Compiling a direct message
 *
 * @package Wikijump\Services\Wikitext
 */
final class ParseRenderMode extends Enum
{
    const FEED = 0;
    const LIST = 1;
    const PAGE = 2;
    const FORUM_POST = 3;
    const DIRECT_MESSAGE = 4;
}
