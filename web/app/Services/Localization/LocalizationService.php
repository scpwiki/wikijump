<?php
declare(strict_types=1);

namespace Wikijump\Services\Localization;


use Illuminate\Support\Facades\App;
use Illuminate\Support\Facades\Log;
use MessageFormatter;

final class LocalizationService
{
    private function __construct()
    {
    }

    /**
     * @return void
     */
    public static function setup()
    {
        // Set up gettext
        bindtextdomain('wikijump', WIKIJUMP_ROOT . '/web/resources/lang');
        bind_textdomain_codeset('wikijump', 'UTF-8');
        textdomain('wikijump');
    }

    /**
     *
     */
    public static function translate(string $key, array $values = []): string
    {
        $locale = App::currentLocale();
        $message = gettext($key);
        if ($message === $key) {
            Log::warning("Unable to find message '$key' in locale '$locale'");
        }

        return MessageFormatter::formatMessage($locale, $message, $values);
    }
}
