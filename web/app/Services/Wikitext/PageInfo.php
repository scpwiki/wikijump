<?php

namespace Wikijump\Services\Wikitext;

/**
 * Class PageInfo, representing information associated with a page for parsing and rendering.
 * @package Wikijump\Services\Wikitext
 */
class PageInfo
{
    public string $page;
    public ?string $category;
    public string $site;
    public string $title;
    public ?string $altTitle;
    public array $tags;
    public string $language;

    public function __construct(
        string $page,
        ?string $category,
        string $site,
        string $title,
        ?string $altTitle,
        array $tags,
        string $language
    ) {
        $this->page = $page;
        $this->category = $category;
        $this->site = $site;
        $this->title = $title;
        $this->altTitle = $altTitle;
        $this->tags = $tags;
        $this->language = $language;
    }
}