<?php

namespace Wikijump\Services\License;


use Exception;

final class LicenseMapping
{
    private static array $mapping = [];

    public static function get(string $id): License
    {
        if (empty(self::$mapping)) {
            self::initialize();
        }

        $license = self::$mapping[$id];
        if ($license === null) {
            throw new Exception("No license with ID $id found");
        }
        return $license;
    }

    private static function initialize()
    {
        $licensesData = config('licenses.raw');

        foreach ($licensesData as &$licenseData) {
            $license = new License($licenseData);
            self::$mapping[$license->id()] = $license;
        }
    }
}
