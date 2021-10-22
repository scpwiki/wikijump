<?php
declare(strict_types=1);

namespace Wikijump\Services\Wikitext;

use Text_Wiki;

/**
 * Class Backlinks, contains information about links found while parsing and rendering.
 * @package Wikijump\Services\Wikitext
 */
class Backlinks
{
    public array $inclusions_present;
    public array $inclusions_absent;
    public array $internal_links_present;
    public array $internal_links_absent;
    public array $external_links;

    public function __construct(
        array $inclusions_present,
        array $inclusions_absent,
        array $internal_links_present,
        array $internal_links_absent,
        array $external_links
    ) {
        $this->inclusions_present = self::dedupeIds($inclusions_present);
        $this->inclusions_absent = self::dedupeStrings($inclusions_absent);
        $this->internal_links_present = self::dedupeIds($internal_links_present);
        $this->internal_links_absent = self::dedupeStrings($internal_links_absent);
        $this->external_links = self::dedupeStrings($external_links);
    }

    private static function dedupeIds(array $items): array
    {
        return array_unique($items, SORT_NUMERIC);
    }

    private static function dedupeStrings(array $items): array
    {
        return array_unique($items, SORT_STRING);
    }

    public static function fromWikiObject(Text_Wiki $wiki): Backlinks
    {
        $inclusions_present = $wiki->vars['inclusions'] ?? [];
        $inclusions_absent = $wiki->vars['inclusionsNotExist'] ?? [];
        $internal_links_present = $wiki->vars['internalLinksExist'] ?? [];
        $internal_links_absent = $wiki->vars['internalLinksNotExist'] ?? [];
        $external_links = $wiki->vars['externalLinks'] ?? [];

        return new Backlinks(
            $inclusions_present,
            $inclusions_absent,
            $internal_links_present,
            $internal_links_absent,
            $external_links,
        );
    }
}
