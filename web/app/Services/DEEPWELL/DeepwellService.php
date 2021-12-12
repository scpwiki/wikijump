<?php
declare(strict_types=1);

namespace Wikijump\Services\DEEPWELL;

use Carbon\Carbon;
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
    public function getUserById(int $id): ?object
    {
        $resp = $this->client->get("user/id/$id");
        if ($resp->getStatusCode() === 404) {
            return null;
        }

        return self::parseUser($resp);
    }

    public function getUserBySlug(string $slug): ?object
    {
        $resp = $this->client->get("user/slug/$slug");
        if ($resp->getStatusCode() === 404) {
            return null;
        }

        return self::parseUser($resp);
    }

    public function setUser(int $id, array $fields): void
    {
        $this->client->put("user/id/$id", ['json' => $fields]);
    }

    private function parseUser($resp): object
    {
        $user = self::readJson($resp);
        $user->email_verified_at = self::nullableDate($user->email_verified_at);
        $user->created_at = self::nullableDate($user->created_at);
        $user->updated_at = self::nullableDate($user->updated_at);
        $user->deleted_at = self::nullableDate($user->deleted_at);
        return $user;
    }

    // Miscellaneous
    public function ping(): void
    {
        $this->client->get('ping');
    }

    public function version(bool $full = false): string
    {
        $method = $full ? 'version/full' : 'version';
        return (string) $this->client->get($method)->getBody();
    }

    // Helper functions

    private static function readJson($resp): object
    {
        $json = (string) $resp->getBody();
        return json_decode($json);
    }

    /**
     * Parses a nullable string and returns a nullable date.
     *
     * @param string|null $value
     * @return Carbon|null
     */
    private static function nullableDate(?string $value): ?Carbon
    {
        return $value ? new Carbon($value) : null;
    }
}
