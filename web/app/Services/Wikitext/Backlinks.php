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
    )
    {
        $this->inclusionsPresent = $inclusionsPresent;
        $this->inclusionsAbsent = $inclusionsAbsent;
        $this->internalLinksPresent = $internalLinksPresent;
        $this->internalLinksAbsent = $internalLinksAbsent;
        $this->externalLinks = $externalLinks;
    }

    public static function fromWikiObject(Text_Wiki $wiki): Backlinks
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
