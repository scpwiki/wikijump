<?php
declare(strict_types=1);

namespace Wikijump\Common;


/**
 * Data class to store a particular configured license.
 * Each must have a unique string ID, which should not change as it is stored in the database.
 *
 * Because these are constants, the fields are exposed via method only to prevent accidental mutation.
 */
final class License
{
    // Mapping
    private static array $mapping;

    // Fields for License instances
    private string $id;
    private string $name;
    private ?string $url;
    private bool $unlessClause;
    private string $html;

    public function __construct(
        string $id,
        string $name,
        ?string $url,
        bool $unlessClause = true
    ) {
        $this->id = $id;
        $this->name = $name;
        $this->url = $url;
        $this->unlessClause = $unlessClause;
        $this->html = $this->buildHtml();
    }

    // TODO: convert this into a blade template
    // These are safe since these objects are built in static configuration
    private function buildHtml(): string
    {
        if ($this->url === null) {
            $link = $this->name;
        } else {
            $link = "<a rel=\"license\" href=\"$this->url\">$this->name</a>";
        }

        if ($this->unlessClause) {
            $unless = __('Unless stated otherwise Content of this page is licensed under');

            return $link . ' ' . $unless;
        } else {
            return $link;
        }
    }

    public function id(): string
    {
        return $this->id;
    }

    public function name(): string
    {
        return $this->name;
    }

    public function url(): ?string
    {
        return $this->url;
    }

    public function hasUnlessClause(): bool
    {
        return $this->unlessClause;
    }
}
