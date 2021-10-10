<?php
declare(strict_types=1);

namespace Wikijump\Services\License;


/**
 * Data class to store a particular configured license.
 * Because these are constants, the fields are exposed via method only to prevent accidental mutation.
 */
final class License
{
    // Fields for License instances
    private string $id;
    private string $name;
    private ?string $url;
    private bool $unlessClause;
    private string $html;

    public function __construct(array &$object) {
        $this->id = $object['id'];
        $this->name = $object['name'];
        $this->url = $object['url'];
        $this->unlessClause = $object['unless'] ?? true;
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

            return $unless . ' ' . $link;
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

    public function html(): string
    {
        return $this->html;
    }

    public static function generateMapping(array &$licenses)
    {
        $ids = [];

        foreach ($licenses as &$license) {
            $ids[$license->id] = $license;
        }

        return $ids;
    }
}
