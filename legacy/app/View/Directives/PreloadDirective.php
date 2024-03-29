<?php

declare(strict_types=1);

namespace Wikijump\View\Directives;

use Illuminate\Support\Facades\Blade;
use Illuminate\Support\Str;
use Wikijump\Common\Asset;

final class PreloadDirective
{
    private function __construct()
    {
    }

    /** Registers the `@preload` directive. */
    public static function register()
    {
        Blade::directive('preload', function ($expression) {
            $namespace = '\Wikijump\View\Directives\PreloadDirective';
            return "<?php echo $namespace::preload({$expression}); ?>";
        });
    }

    /**
     * Returns an HTML tag for a preload link.
     *
     * @param string $href The URL to preload.
     * @param string $as The type of resource to preload.
     * @param string $type The MIME type to preload. (optional)
     * @param bool $crossorigin Whether to allow crossorigin requests. (optional)
     */
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

    /**
     * Returns a string of potentially multiple link preload tags,
     * depending on the path given.
     *
     * @param string $path The path to preload.
     */
    public static function preload(string $path): string
    {
        $urls = (new Asset($path))->urls();

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
