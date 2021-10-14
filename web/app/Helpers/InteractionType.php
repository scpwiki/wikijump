<?php
declare(strict_types=1);

namespace Wikijump\Helpers;

use Wikijump\Common\Enum;

/**
 * Enumerating possible interaction types for use with the Interaction class.
 */
final class InteractionType extends Enum
{
    const USER_FOLLOWS_USER = 0;
    const USER_FOLLOWS_PAGE = 1;
    const USER_FOLLOWS_FORUM = 2;
    const USER_FOLLOWS_FORUM_THREAD = 3;
    const USER_FOLLOWS_SITE = 4;
    const USER_BLOCKS_USER = 5;
    const SITE_BLOCKS_USER_READS = 6;
    const SITE_BLOCKS_USER_WRITES = 7;
    const SITE_BLOCKS_USER_APPLICATIONS = 8;
    const PAGE_BLOCKS_USER_READS = 9;
    const PAGE_BLOCKS_USER_WRITES = 10;
    const FORUM_BLOCKS_USER_READS = 12;
    const FORUM_BLOCKS_USER_WRITES = 12;
    const FORUM_THREAD_BLOCKS_USER_READS = 13;
    const FORUM_THREAD_BLOCKS_USER_WRITES = 14;
    const USER_CONTACT_REQUESTS = 15;
    const USER_CONTACTS = 16;
    const USER_FAVORITES_PAGE = 17;
    const USER_FAVORITES_FORUM = 18;
    const USER_FAVORITES_FORUM_THREAD = 19;
    const USER_FAVORITES_SITE = 20;
}
