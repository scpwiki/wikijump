<?php
declare(strict_types=1);

namespace Wikijump\Services\Localization;

use Illuminate\Support\Facades\App;
use Illuminate\Support\Facades\Log;
use Gettext\Loader\MoLoader;
use Gettext\Translations;
use MessageFormatter;

final class LocalizationService
{
    public const LOCALES_DIRECTORY = WIKIJUMP_ROOT . '/public/files--built/locales/';
    private static Translations $translations;

    public static function setup(): void
    {
        $locale = App::currentLocale();
        self::$translations = self::loadTranslations($locale);
    }

    public static function loadTranslations(string $locale): Translations
    {
        $loader = new MoLoader();
        $path = self::LOCALES_DIRECTORY . $locale . '.mo';
        return $loader->loadFile($path);
    }

    public static function translate(string $key, array $values = []): string
    {
        if ($key === '') {
            Log::error('Empty localization key given');
            return '';
        }

        // Get message from translations file
        $locale = App::currentLocale();
        $translation = self::$translations->find(null, $key);
        if ($translation === null) {
            Log::warning('Unable to find message', ['key' => $key, 'locale' => $locale]);
            return $key;
        }

        // Format ICU localization message
        $message = $translation->getTranslation();
        $output = MessageFormatter::formatMessage($locale, $message, $values);
        Log::debug('Translated message', ['key' => $key, 'output' => $output]); // TODO: remove this
        return $output;
    }
}
