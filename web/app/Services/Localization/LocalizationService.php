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
     * Sets up the environment for localization calls.
     * @return void
     */
    public static function setup()
    {
        // Set up gettext
        bindtextdomain('wikijump', WIKIJUMP_ROOT . '/web/public/locales');
        bind_textdomain_codeset('wikijump', 'UTF-8');
        textdomain('wikijump');
    }

    public static function translate(string $key, array $values = []): string
    {
        // Set locale for gettext
        $locale = App::currentLocale();
        setlocale(LC_MESSAGES, $locale);

        // Get appropriate string
        $message = gettext($key);
        if ($message === $key) {
            Log::warning("Unable to find message '$key' in locale '$locale'");
        }

        // Format ICU localization message
        return MessageFormatter::formatMessage($locale, $message, $values);
    }
}
