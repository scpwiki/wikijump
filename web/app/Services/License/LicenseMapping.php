<?php

namespace Wikijump\Services\License;

use Exception;

final class LicenseMapping
{
    private static bool $initialized = false;
    private static array $ordered = [];
    private static array $mapping = [];

    private static function initialize(): void
    {
        if (self::$initialized) {
            return;
        }

        $licensesData = config('licenses.raw');
        foreach ($licensesData as &$licenseData) {
            $license = new License($licenseData);
            array_push(self::$ordered, $license);
            self::$mapping[$license->id()] = $license;
        }
        self::$initialized = true;
    }

    public static function get(string $id): License
    {
        self::initialize();
        $license = self::$mapping[$id];
        if ($license === null) {
            throw new Exception("No license with ID $id found");
        }
        return $license;
    }

    public static function list(): array
    {
        self::initialize();
        return self::$ordered;
    }
}
