<?php

declare(strict_types=1);

namespace Wikijump\Services\Nginx;

use Exception;
use Illuminate\Support\Facades\Log;

/**
 * Static class that holds methods for interacting with the NGINX container.
 */
final class Nginx
{
    // TODO: Docker paths aren't reliable (or working at all) on AWS
    private const NGINX_URL = 'http://nginx:80/';

    private function __construct()
    {
    }

    /**
     * Fires a request at the path given, returning the response as a string.
     * Will return null if the request fails.
     *
     * @param string $path Path to request, usually a file.
     * @param string $accept Accept header to send.
     */
    public static function fetch(string $path, string $accept = 'text/plain'): ?string
    {
        $url = self::NGINX_URL . $path;

        $opts = [
            'http' => [
                'method' => 'GET',
                'header' => "Accept: {$accept}",
            ],
        ];

        try {
            $context = stream_context_create($opts);
            $contents = file_get_contents($url, false, $context);

            if ($contents === false) {
                return null;
            }

            return $contents;
        } catch (Exception $err) {
            Log::error("Could not fetch asset contents: {$url}");
            Log::error($err->getMessage());
            Log::notice("(Note: If the instance is still starting up, this error can likely be ignored)");
            return null;
        }
    }
}
