<?php
declare(strict_types=1);

namespace Wikijump\Common;


/**
 * Data class to store a particular configured license.
 *
 * Because these are constants, the fields are exposed via method only to prevent accidental mutation.
 */
final class License
{
    private string $name;
    private string $url;
    private bool $unlessClause;
    private string $html;

    public function __construct(
        string $name,
        string $url,
        bool $unlessClause = true
    ) {
        $this->name = $name;
        $this->url = $url;
        $this->unlessClause = $unlessClause;
        $this->html = $this->buildHtml();
    }

    // TODO: convert this into a blade template
    // These are safe since these objects are built in static configuration
    private function buildHtml(): string
    {
        $link = "<a rel=\"license\" href=\"$this->url\">$this->name</a>";

        if ($this->unlessClause) {
            $unless = __('Unless stated otherwise Content of this page is licensed under');

            return $link . ' ' . $unless;
        } else {
            return $link;
        }
    }

    public function name(): string
    {
        return $this->name;
    }

    public function url(): string
    {
        return $this->url;
    }

    public function hasUnlessClause(): bool
    {
        return $this->unlessClause;
    }
}
