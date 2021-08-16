<?php
declare(strict_types=1);

namespace Wikijump\Services\Wikitext;

/**
 * Class PageRef, refers to a particular page by slug. If the site is null, then the current site is used.
 * @package Wikijump\Services\Wikitext
 */
class PageRef
{
    public ?string $site;
    public string $page;

    public function __construct(?string $site, string $page)
    {
        $this->site = $site;
        $this->page = $page;
    }

    public function pathRepr(): string
    {
        if ($this->site === null) {
            $site = '';
        } else {
            $site = ":$this->site:";
        }

        return $site . $this->page;
    }
}
