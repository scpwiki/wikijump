<?php

declare(strict_types=1);

namespace Wikijump\Common;

use Exception;
use Illuminate\Support\Facades\App;
use Illuminate\Support\Str;
use Innocenzi\Vite\ManifestEntry;
use Innocenzi\Vite\Vite;
use Wikijump\Services\Nginx\Nginx;

/**
 * Utility for resolving and working with assets by their unresolved path.
 */
class Asset
{
    /** Name of the asset. */
    private string $name;

    /** Unresolved path to the asset. */
    private string $path;

    /** Resolved path to the asset. */
    private ?string $resolved_path;

    /** Entry for the asset in the Vite manifest. */
    private ?ManifestEntry $manifest_entry;

    /** Path added to entrypoints not given with an explicit path. */
    private const ENTRYPOINT_FOLDER = 'resources/scripts/';

    /**
     * @param string $name Name or path of the asset.
     */
    public function __construct(string $name)
    {
        $this->name = $name;

        // if the path doesn't start with a `/`, it's a Vite path
        if (!Str::startsWith($name, '/')) {
            $vite = App::make(Vite::class);

            // check if path has any slashes in it
            $is_shorthand = strpos($name, '/') === false;

            if ($is_shorthand) {
                $this->path = self::ENTRYPOINT_FOLDER . $name;
            } else {
                $this->path = $name;
            }

            if ($vite->isDevelopmentServerRunning()) {
                $this->resolved_path = config('vite.dev_url') . '/' . $this->path;
            } else {
                $entry = $vite->getEntries()->get($this->path);

                if (!$entry) {
                    throw new Exception("Asset not found: {$this->path}");
                }

                $this->manifest_entry = $entry;

                $this->resolved_path = asset(
                    sprintf('/%s/%s', config('vite.build_path'), $entry->file),
                );

                $this->name = basename($entry->file);
            }
        }
        // starts with `/`, treat as a static path
        else {
            $this->path = $name;
            $this->name = basename($name);
        }
    }

    /**
     * Returns the name of the asset, as in their filename.
     *
     * @param bool $with_extension If false, the extension will be removed from the asset.
     */
    public function name(bool $with_extension = true): string
    {
        if ($with_extension) {
            return $this->name;
        }

        $extension_pos = strrpos($this->name, '.');

        if ($extension_pos === false) {
            return $this->name;
        }

        return substr($this->name, 0, $extension_pos);
    }

    /** Returns the extension of the asset, without the `.` at the start. */
    public function extension(): string
    {
        $extension_pos = strrpos($this->name, '.');

        if ($extension_pos === false) {
            return '';
        }

        return substr($this->name, $extension_pos + 1);
    }

    /** Returns true if the asset is a script. */
    public function isScript(): bool
    {
        $ext = $this->extension();
        return $ext === 'js' || $ext === 'ts';
    }

    /** Returns true if the asset is a stylesheet. */
    public function isStylesheet(): bool
    {
        $ext = $this->extension();
        return $ext === 'css' || $ext === 'scss';
    }

    /** Returns the unresolved path to the asset. */
    public function unresolvedPath(): string
    {
        return $this->path;
    }

    /** Returns the resolved path to the asset, if different from the unresolved path. */
    public function path(): string
    {
        return $this->resolved_path ?? $this->path;
    }

    /** Returns an array of strings representing the CSS imports made by the asset. */
    public function css(): array
    {
        $urls = [];

        if (empty($this->manifest_entry)) {
            return $urls;
        }

        $this->manifest_entry->css->each(function (string $path) use (&$urls) {
            $urls[] = asset(sprintf('/%s/%s', config('vite.build_path'), $path));
        });

        return $urls;
    }

    /** Returns an array of strings representing the JS imports made by the assets. */
    public function imports(): array
    {
        // TODO: laravel-vite can't keep track of imports yet
        return [];
    }

    /** Returns all URLs relevant to the asset, including its own path. */
    public function urls(): array
    {
        return array_merge([$this->path()], $this->imports(), $this->css());
    }

    /** Returns the contents of the asset as a string. */
    public function contents(): string
    {
        $accept_type = $this->isScript()
            ? 'text/javascript'
            : ($this->isStylesheet()
                ? 'text/css'
                : 'text/plain');

        $contents = Nginx::fetch($this->path(), $accept_type);

        if ($contents === null) {
            throw new Exception("Could not read asset contents: {$this->path}");
        }

        return $contents;
    }
}
