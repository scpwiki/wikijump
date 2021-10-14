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
    public array $inclusionsPresent;
    public array $inclusionsAbsent;
    public array $internalLinksPresent;
    public array $internalLinksAbsent;
    public array $externalLinks;

    public function __construct(
        array $inclusionsPresent,
        array $inclusionsAbsent,
        array $internalLinksPresent,
        array $internalLinksAbsent,
        array $externalLinks
    ) {
        $this->inclusionsPresent = self::dedupeIds($inclusionsPresent);
        $this->inclusionsAbsent = self::dedupeStrings($inclusionsAbsent);
        $this->internalLinksPresent = self::dedupeIds($internalLinksPresent);
        $this->internalLinksAbsent = self::dedupeStrings($internalLinksAbsent);
        $this->externalLinks = self::dedupeStrings($externalLinks);
    }

    private static function dedupeIds(array &$items): array
    {
        return array_unique($items, SORT_NUMERIC);
    }

    private static function dedupeStrings(array &$items): array
    {
        return array_unique($items, SORT_STRING);
    }

    public static function fromWikiObject(Text_Wiki &$wiki): Backlinks
    {
        $inclusionsPresent = $wiki->vars['inclusions'] ?? [];
        $inclusionsAbsent = $wiki->vars['inclusionsNotExist'] ?? [];
        $internalLinksPresent = $wiki->vars['internalLinksExist'] ?? [];
        $internalLinksAbsent = $wiki->vars['internalLinksNotExist'] ?? [];
        $externalLinks = $wiki->vars['externalLinks'] ?? [];

        return new Backlinks(
            $inclusionsPresent,
            $inclusionsAbsent,
            $internalLinksPresent,
            $internalLinksAbsent,
            $externalLinks,
        );
    }
}
