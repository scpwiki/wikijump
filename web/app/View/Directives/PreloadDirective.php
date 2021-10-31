<?php

declare(strict_types=1);

namespace Wikijump\View\Directives;

use Illuminate\Support\Facades\App;
use Illuminate\Support\Facades\Blade;
use Illuminate\Support\Str;
use Innocenzi\Vite\Vite;

final class PreloadDirective
{
    private function __construct()
    {
    }

    public static function register()
    {
        Blade::directive('preload', function ($expression) {
            $namespace = '\Wikijump\View\Directives\PreloadDirective';
            return "<?php echo $namespace::preload({$expression}); ?>";
        });
    }

    public static function preloadTag(
        string $href,
        string $as,
        string $type = '',
        bool $crossorigin = false
    ): string {
        $crossorigin = $crossorigin ? 'crossorigin' : '';
        $type = $type ? "type=\"{$type}\"" : '';
        $rel = Str::endsWith($href, ['.js', '.ts']) ? 'modulepreload' : 'preload';
        return "<link rel=\"$rel\" href=\"$href\" as=\"$as\" $type $crossorigin />\n";
    }

    public static function preload(string $path): string
    {
        $urls = [];

        // if the URL doesn't start with a slash, we'll retrieve it from the manifest
        if (!Str::startsWith($path, '/')) {
            $vite = App::make(Vite::class);

            // generate from development url
            if (App::environment('local') && $vite->isDevelopmentServerRunning()) {
                $urls[] = config('vite.dev_url') . '/' . $path;
            }
            // generate from a manifest entry
            else {
                $entry = $vite->getEntries()->get($path);
                if ($entry) {
                    $urls[] = asset(
                        sprintf('/%s/%s', config('vite.build_path'), $entry->file),
                    );

                    // TODO: preload imports, too
                    // laravel-vite doesn't keep track of imports from the manifest,
                    // so this isn't possible yet

                    // preload imported CSS
                    $entry->css->each(function (string $path) use (&$urls) {
                        $urls[] = asset(
                            sprintf('/%s/%s', config('vite.build_path'), $path),
                        );
                    });
                }
            }
        } else {
            $urls[] = $path;
        }

        $html = '';

        foreach ($urls as $url) {
            // file extension (no . at the beginning)
            $ext = pathinfo($url, PATHINFO_EXTENSION);

            // prettier-ignore
            switch ($ext) {
                case 'ts':    $html .= self::preloadTag($url, 'script'); break;
                case 'js':    $html .= self::preloadTag($url, 'script'); break;
                case 'scss':  $html .= self::preloadTag($url, 'style'); break;
                case 'css':   $html .= self::preloadTag($url, 'style'); break;
                case 'woff2': $html .= self::preloadTag($url, 'font', 'font/woff2', true); break;
            }
        }

        return $html;
    }
}
