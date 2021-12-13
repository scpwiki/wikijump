<?php

declare(strict_types=1);

namespace Wikijump\View\Directives;

use Illuminate\Support\Facades\Blade;
use Wikijump\Common\Asset;

final class InlineDirective
{
    private function __construct()
    {
    }

    /** Registers the `@inline` directive. */
    public static function register()
    {
        Blade::directive('inline', function ($expression) {
            $namespace = '\Wikijump\View\Directives\InlineDirective';
            return "<?php echo $namespace::inline({$expression}); ?>";
        });
    }

    /**
     * Returns the HTML for inlining a style or script tag.
     * @param string $path The path to the asset.
     */
    public static function inline($path): string
    {
        $asset = new Asset($path);
        $extension = $asset->extension();
        $contents = $asset->contents();

        switch ($extension) {
            case 'ts':
            case 'js':
                return "<script type=\"module\">$contents</script>";
            case 'scss':
            case 'css':
                return "<style>$contents</style>";
        }
    }
}
