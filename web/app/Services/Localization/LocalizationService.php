<?php
declare(strict_types=1);

namespace Wikijump\Services\Localization;

use Illuminate\Support\Facades\App;
use Illuminate\Support\Facades\Log;
use Wikijump\Services\Deepwell\DeepwellService;

/**
 * Service to provide formatted translated messages depending on the current locale.
 */
final class LocalizationService
{
    public static function translate(string $key, array $values = []): string
    {
        if ($key === '') {
            Log::error('Empty localization key given');
            return '';
        }

        $locale = App::currentLocale();
        $translation = DeepwellService::getInstance()->translate($locale, $key, $values);
        if ($translation === null) {
            Log::warning('Unable to find message', ['key' => $key, 'locale' => $locale]);
            return $key;
        }

        return $translation;
    }
}
