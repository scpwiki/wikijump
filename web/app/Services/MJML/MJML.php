<?php

declare(strict_types=1);

namespace Wikijump\Services\MJML;

// TODO: use mrml-cli when compile bug is fixed

// use Symfony\Component\Process\Exception\ProcessFailedException;
// use Symfony\Component\Process\Process;

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
    public static function render(string $template_path, array $data = []): string
    {
        $view = view($template_path, $data);
        $raw_mjml = $view->render();

        // // execute mrml-cli from shell to convert to html
        // $proc = Process::fromShellCommandline('mrml-cli', null, null, $raw_mjml);

        // $proc->run();

        // if (!$proc->isSuccessful()) {
        //     throw new ProcessFailedException($proc);
        // }

        // return $proc->getOutput();

        return $raw_mjml;
    }
}
