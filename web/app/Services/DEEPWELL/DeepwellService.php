<?php
declare(strict_types=1);

namespace Wikijump\Services\DEEPWELL;

use GuzzleHttp\Client;

final class DeepwellService
{
    private static ?DeepwellService $instance = null;

    public static function getInstance(): DeepwellService
    {
        if (self::$instance === null) {
            self::$instance = new DeepwellService();
        }

        return self::$instance;
    }

    private Client $client;

    private function __construct()
    {
        $this->client = new Client([
            'base_uri' => 'http://api:2747/api/v0/',
            'timeout' => 1.0,
        ]);
    }

    // User
    // TODO

    // Miscellaneous
    public function ping(): void
    {
        $this->client->get('ping');
    }

    public function version(bool $full = false): string
    {
        $method = $full ? 'version/full' : 'version';
        return (string)$this->client->get($method)->getBody();
    }
}
