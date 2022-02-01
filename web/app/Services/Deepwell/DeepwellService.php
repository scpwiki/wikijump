<?php
declare(strict_types=1);

namespace Wikijump\Services\Deepwell;

use Carbon\Carbon;
use GuzzleHttp\Client;
use GuzzleHttp\Exception\ClientException;
use GuzzleHttp\Exception\GuzzleException;
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
            'base_uri' => 'http://api:2747/api/vI/',
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
            $resp = $this->client->get("message/$locale/$key", [
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
    public function getPageById(int $site_id, int $page_id): ?Page
    {
        return self::fetchOrNull(function () use ($site_id, $page_id) {
            $resp = $this->client->get("page/$site_id/id/$page_id");
            return $this->parsePage($resp);
        }, "No page found in site ID $site_id with page ID $page_id");
    }

    public function getPageBySlug(int $site_id, string $page_slug): ?Page
    {
        return self::fetchOrNull(function () use ($site_id, $page_slug) {
            $resp = $this->client->get("page/$site_id/slug/$page_slug");
            return $this->parsePage($resp);
        }, "No page found in site ID $site_id with slug '$page_slug'");
    }

    public function editPage(
        int $site_id,
        int $page_id,
        int $user_id,
        string $comments,
        array $changes
    ): void {
        $change_fields = [
            'wikitext' => null,
            'title' => null,
            'alt_title' => 'altTitle',
            'tags' => null,
        ];

        $body = [
            'revisionComments' => $comments,
            'userId' => $user_id,
        ];

        foreach ($change_fields as $php_name => $json_name) {
            $json_name ??= $php_name;
            if (array_key_exists($php_name, $changes)) {
                $body[$json_name] = $changes[$php_name];
            }
        }

        $this->client->post("page/$site_id/id/$page_id", ['json' => $body]);
    }

    public function deletePage(
        int $site_id,
        int $page_id,
        int $user_id,
        string $comments
    ): void {
        $this->client->delete("page/$site_id/id/$page_id", [
            'json' => [
                'revisionComments' => $comments,
                'userId' => $user_id,
            ],
        ]);
    }

    public function undeletePage(
        int $site_id,
        int $page_id,
        int $user_id,
        string $comments
    ): void {
        $this->client->post("page/$site_id/$page_id", [
            'json' => [
                'revisionComments' => $comments,
                'userId' => $user_id,
            ],
        ]);
    }

    public function rerenderPage(
        int $site_id,
        int $page_id,
        int $user_id,
        string $comments
    ): void {
        $this->client->post("page/$site_id/$page_id", [
            'json' => [
                'revisionComments' => $comments,
                'userId' => $user_id,
            ],
        ]);
    }

    public function getLatestPageRevision(int $site_id, int $page_id): object
    {
        return self::parsePageRevision(
            $this->client->get("page/$site_id/id/$page_id/revision"),
        );
    }

    public function getPageRevision(
        int $site_id,
        int $page_id,
        int $revision_number
    ): ?object {
        return self::fetchOrNull(function () use ($site_id, $page_id, $revision_number) {
            $resp = $this->client->get(
                "page/$site_id/id/$page_id/revision/$revision_number",
            );
            return self::parsePageRevision($resp);
        }, "No page revision $revision_number found for page ID $page_id in site ID $site_id");
    }

    public function setPageRevision(
        int $site_id,
        int $page_id,
        int $revision_number,
        int $user_id,
        array $hidden
    ): void {
        $this->client->put("page/$site_id/id/$page_id/revision/$revision_number", [
            'json' => [
                'userId' => $user_id,
                'hidden' => $hidden,
            ],
        ]);
    }

    public function getLinksFrom(int $site_id, int $page_id): array
    {
        $resp = $this->client->get("page/$site_id/id/$page_id/links/from");
        return self::readJson($resp);
    }

    /**
     * @param string|int $site_id
     * @param string|int $page_id
     * @throws GuzzleException
     */
    public function getLinksTo($site_id, $page_id): array
    {
        $resp = $this->client->get("page/$site_id/id/$page_id/links/to");
        return self::readJson($resp);
    }

    /**
     * @param string|int $site_id
     * @throws GuzzleException
     */
    public function getLinksToMissing($site_id, string $page_slug): array
    {
        $resp = $this->client->get("page/$site_id/slug/$page_slug/links/to/missing");
        return self::readJson($resp);
    }

    private static function backlinksToJson(Backlinks $backlinks): array
    {
        // TODO in ftml, this struct uses kebab-case
        return [
            'included-pages' => $backlinks->inclusions,
            'internal-links' => $backlinks->internal_links,
            'external-links' => $backlinks->external_links,
        ];
    }

    // Text
    // TEMP!
    public function getText(string $hex_hash): string
    {
        $resp = $this->client->get("text/$hex_hash");
        return (string) $resp->getBody();
    }

    public function addText(string $contents): string
    {
        $resp = $this->client->put('text', [
            'body' => $contents,
        ]);

        return (string) $resp->getBody();
    }

    // User
    public function getUserById(int $id, string $detail = 'identity'): ?User
    {
        return self::fetchOrNull(function () use ($id, $detail) {
            $resp = $this->client->get("user/id/$id", [
                'query' => ['detail' => $detail],
            ]);

            return $this->parseUser($resp);
        }, "No user found with ID $id");
    }

    public function getUserBySlug(string $slug, string $detail = 'string'): ?User
    {
        return self::fetchOrNull(function () use ($slug, $detail) {
            $resp = $this->client->get("user/id/$slug", [
                'query' => ['detail' => $detail],
            ]);

            return $this->parseUser($resp, $detail);
        }, "No user found with slug $slug");
    }

    public function setUser(int $id, array $fields): void
    {
        $this->client->put("user/id/$id", ['json' => $fields]);
    }

    private function parsePage($resp): Page
    {
        $page = self::readJson($resp);
        self::convertDateProperties($page, ['createdAt', 'updatedAt', 'deletedAt']);
        return new Page($page);
    }

    private function parsePageRevision($resp): PageRevision
    {
        $revision = self::readJson($resp);
        self::convertDateProperties($revision, ['createdAt', 'compiledAt']);
        return new PageRevision($revision);
    }

    private function parseUser($resp, string $detail): User
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

        return new User($user, $detail);
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

    /**
     * Runs the given closure.
     * If it succeeds, pass the result on.
     * If it yields HTTP 404, then return null.
     *
     * @param callable $fn The closure to run.
     * @param ?string $warning The message to log if the item is not found. Doesn't log if null.
     * @return ?mixed
     */
    private static function fetchOrNull(callable $fn, ?string $warning = null)
    {
        try {
            return $fn();
        } catch (ClientException $exception) {
            if ($exception->getCode() === 404) {
                Log::warning($warning);
                return null;
            }

            // For other errors, re-throw, since it's not an "expected" error
            throw $exception;
        }
    }
}
