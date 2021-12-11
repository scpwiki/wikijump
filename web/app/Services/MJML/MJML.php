<?php

declare(strict_types=1);

namespace Wikijump\Services\MJML;

use Illuminate\Support\Facades\Cache;
use Illuminate\Support\HtmlString;
use Symfony\Component\Process\Exception\ProcessFailedException;
use Symfony\Component\Process\Process;

/** Static class holding methods for handling MJML templates. */
final class MJML
{
    private function __construct()
    {
    }

    /**
     * Compiles a MJML-based Blade template into HTML.
     *
     * @param string $template_path Blade template path.
     * @param array $data Data to be passed to the template.
     */
    public static function render(string $template_path, array $data = []): HtmlString
    {
        $view = view($template_path, $data);
        $raw_mjml = $view->render();

        // we use the hash of the raw MJML, because Blade templates already
        // have caching built in, so we'll reuse that machinery rather than remaking it
        $hash = hash('sha256', $raw_mjml);

        // cache hit, return the cached HTML
        if (Cache::has($hash)) {
            return new HtmlString(Cache::get($hash));
        }

        // execute mrml-cli from shell to convert to html
        // command has a 5 second timeout
        // if rendering takes longer than that, something is very wrong
        // average render time should be in milliseconds
        $proc = Process::fromShellCommandline('mrml render', null, null, $raw_mjml, 5);

        $proc->run();

        if (!$proc->isSuccessful()) {
            throw new ProcessFailedException($proc);
        }

        $html = $proc->getOutput();

        // cache the result
        // we don't use a particularly long cache, because most emails are personalized
        // caching is mostly for caching repeated emails, like marketing emails
        Cache::put($hash, $html, now()->addMinutes(5));

        return new HtmlString($html);
    }
}
