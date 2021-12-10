<?php

declare(strict_types=1);

namespace Wikijump\Services\MJML;

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

        // TODO: add caching

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

        return new HtmlString($html);
    }
}
