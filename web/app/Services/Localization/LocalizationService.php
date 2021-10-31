<?php
declare(strict_types=1);

namespace Wikijump\Services\Localization;

use Illuminate\Support\Facades\App;
use Illuminate\Support\Facades\Log;
use Gettext\Loader\MoLoader;
use Gettext\Translations;
use MessageFormatter;

/**
 * Service to provide formatted translated messages depending on the current locale.
 */
final class LocalizationService
{
    public const LOCALES_DIRECTORY = WIKIJUMP_ROOT . '/public/files--built/locales/';
    private static ?Translations $translations = null;

    private static function loadTranslations(): void
    {
        if (self::$translations === null) {
            $locale = App::currentLocale();
            $loader = new MoLoader();
            $path = self::LOCALES_DIRECTORY . $locale . '.mo';
            self::$translations = $loader->loadFile($path);
        }
    }

    public static function translate(string $key, array $values = []): string
    {
        self::loadTranslations();

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
        return $output;
    }
}
