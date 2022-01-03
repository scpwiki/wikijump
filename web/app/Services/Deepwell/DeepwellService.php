<?php
declare(strict_types=1);

namespace Wikijump\Services\Deepwell;

use Carbon\Carbon;
use GuzzleHttp\Client;
use GuzzleHttp\Exception\ClientException;
use Illuminate\Support\Facades\Log;
use Wikidot\Utils\GlobalProperties;
use Wikijump\Services\Wikitext\Backlinks;

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
            'timeout' => 0.5,
            'headers' => [
                'User-Agent' => 'wikijump-php',
                'X-Exempt-RateLimit' => GlobalProperties::$API_RATELIMIT_BYPASS,
            ],
        ]);

        $this->checkRatelimitExempt();
    }

    // Localization
    public function parseLocale(string $locale): object
    {
        $resp = $this->client->get("locale/$locale");
        return self::readJson($resp);
    }

    public function translate(string $locale, string $key, array $values = []): ?string
    {
        try {
            $resp = $this->client->put("message/$locale/$key", [
                'body' => json_encode($values, JSON_FORCE_OBJECT),
            ]);

            return (string) $resp->getBody();
        } catch (ClientException $exception) {
            // TOOO remove this and have it just throw, after we've removed all the old Wikidot strings

            if ($exception->getCode() === 404) {
                Log::warning("Error retrieving translation: {$exception->getMessage()}");
                return null;
            }

            // For other errors, re-throw, since it's not an "expected" error
            throw $exception;
        }
    }

    // Page
    public function getLinksFrom(int $site_id, int $page_id): array
    {
        $resp = $this->client->get("page/$site_id/id/$page_id/links/from");
        return self::readJson($resp);
    }

    /**
     * @param string|int $site_id
     * @param string|int $page_id
     */
    public function getLinksTo($site_id, $page_id): array
    {
        $resp = $this->client->get("page/$site_id/id/$page_id/links/to");
        return self::readJson($resp);
    }

    /**
     * @param string|int $site_id
     */
    public function getLinksToMissing($site_id, string $page_slug): array
    {
        $resp = $this->client->get("page/$site_id/slug/$page_slug/links/to/missing");
        return self::readJson($resp);
    }

    // TEMP!
    public function updateLinks(
        string $site_id,
        string $page_id,
        Backlinks $backlinks
    ): void {
        $this->client->put("page/$site_id/id/$page_id/links", [
            'json' => self::backlinksToJson($backlinks),
        ]);
    }

    // TEMP!
    public function updateLinksMissing(
        string $site_id,
        string $page_slug,
        Backlinks $backlinks
    ): void {
        $this->client->put("page/$site_id/$page_slug/links/missing", [
            'json' => self::backlinksToJson($backlinks),
        ]);
    }

    private static function backlinksToJson(Backlinks $backlinks): array
    {
        return [
            'included_pages' => $backlinks->inclusions,
            'internal_links' => $backlinks->internal_links,
            'external_links' => $backlinks->external_links,
        ];
    }

    // User
    public function getUserById(int $id, string $detail = 'identity'): ?object
    {
        $resp = $this->client->get("user/id/$id", [
            'query' => ['detail' => $detail],
        ]);

        if ($resp->getStatusCode() === 404) {
            return null;
        }

        return $this->parseUser($resp);
    }

    public function getUserBySlug(string $slug, string $detail = 'string'): ?object
    {
        $resp = $this->client->get("user/id/$slug", [
            'query' => ['detail' => $detail],
        ]);

        if ($resp->getStatusCode() === 404) {
            return null;
        }

        return $this->parseUser($resp);
    }

    public function setUser(int $id, array $fields): void
    {
        $this->client->put("user/id/$id", ['json' => $fields]);
    }

    private function parseUser($resp): object
    {
        $user = self::readJson($resp);

        self::convertDateProperties($user, [
            'since',
            'lastActive',
            'birthday',
            'email_verified_at',
            'created_at',
            'updated_at',
            'deleted_at',
        ]);

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

    public function checkRatelimitExempt(): void
    {
        $this->client->get('ratelimit-exempt');
    }

    // Helper functions

    private static function readJson($resp): object
    {
        $json = (string) $resp->getBody();
        return json_decode($json);
    }

    /**
     * Converts properties in an object to nullable dates, for each property that exists.
     * @param object $obj The object to convert the properties in.
     * @param array $properties The properties to convert.
     */
    private static function convertDateProperties(object $obj, array $properties): void
    {
        foreach ($properties as $property) {
            if (property_exists($obj, $property)) {
                $obj->$property = self::nullableDate($obj->$property);
            }
        }
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
