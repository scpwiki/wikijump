<?php
declare(strict_types=1);

namespace Wikijump\Services\Deepwell;

use Carbon\Carbon;
use GuzzleHttp\Client;
use GuzzleHttp\Exception\ClientException;
use GuzzleHttp\Exception\GuzzleException;
use Illuminate\Support\Facades\Log;
use Wikidot\Utils\GlobalProperties;
use Wikijump\Services\Deepwell\Models\Category;
use Wikijump\Services\Deepwell\Models\Page;
use Wikijump\Services\Wikitext\ParseRenderMode;
use Wikijump\Services\Deepwell\Models\User;

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

    // Rendering
    public function renderHtml(
        ParseRenderMode $mode,
        string $wikitext,
        ?PageInfo $page_info
    ): string {
        // TODO stub
        return "!! TODO !! $wikitext";
    }

    // Category
    public function getCategoryBySlug(int $site_id, string $category_slug): ?Category
    {
        return self::fetchOrNull(function () use ($site_id, $category_slug) {
            $resp = $this->client->get("category/$site_id/slug/$category_slug");
            return $this->parseCategory($resp);
        });
    }

    public function getCategoryById(int $site_id, int $category_id): ?Category
    {
        return self::fetchOrNull(function () use ($site_id, $category_id) {
            $resp = $this->client->get("category/$site_id/id/$category_id");
            return $this->parseCategory($resp);
        });
    }

    public function getCategoryByIdOnly(int $category_id): ?Category
    {
        return self::fetchOrNull(function () use ($category_id) {
            $resp = $this->client->get("category/direct/$category_id");
            return $this->parseCategory($resp);
        });
    }

    public function getCategories(int $site_id): array
    {
        $resp = $this->client->get("category/$site_id");
        return array_map(function (object $raw_category) {
            self::convertDateProperties($raw_category, ['createdAt', 'updatedAt']);
            return new Category($raw_category);
        }, self::readJson($resp));
    }

    // Page
    public function getPageBySlug(
        int $site_id,
        string $page_slug,
        bool $wikitext = false,
        bool $compiledHtml = false
    ): ?Page {
        return self::fetchOrNull(function () use (
            $site_id,
            $page_slug,
            $wikitext,
            $compiledHtml
        ) {
            $resp = $this->client->get("page/$site_id/slug/$page_slug", [
                'query' => [
                    'wikitext' => self::booleanValue($wikitext),
                    'compiledHtml' => self::booleanValue($compiledHtml),
                ],
            ]);
            return $this->parsePage($resp);
        },
        "No page found in site ID $site_id with slug '$page_slug'");
    }

    public function getPageById(
        int $site_id,
        int $page_id,
        bool $wikitext = false,
        bool $compiledHtml = false
    ): ?Page {
        return self::fetchOrNull(function () use (
            $site_id,
            $page_id,
            $wikitext,
            $compiledHtml
        ) {
            $resp = $this->client->get("page/$site_id/id/$page_id", [
                'query' => [
                    'wikitext' => self::booleanValue($wikitext),
                    'compiledHtml' => self::booleanValue($compiledHtml),
                ],
            ]);
            return $this->parsePage($resp);
        },
        "No page found in site ID $site_id with page ID $page_id");
    }

    public function getPageByIdOnly(
        int $page_id,
        bool $wikitext = false,
        bool $compiledHtml = false
    ): ?Page {
        return self::fetchOrNull(function () use ($page_id, $wikitext, $compiledHtml) {
            $resp = $this->client->get("page/direct/$page_id", [
                'query' => [
                    'wikitext' => self::booleanValue($wikitext),
                    'compiledHtml' => self::booleanValue($compiledHtml),
                ],
            ]);
            return $this->parsePage($resp);
        }, "No page found with page ID $page_id");
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

    /**
     * @param string|int $site_id
     * @param string|int $page_id
     * @throws GuzzleException
     */
    public function getLinksTo($site_id, $page_id): object
    {
        $resp = $this->client->get("page/$site_id/id/$page_id/links/to");
        return self::readJson($resp);
    }

    /**
     * @param string|int $site_id
     * @throws GuzzleException
     */
    public function getLinksToMissing($site_id, string $page_slug): object
    {
        $resp = $this->client->get("page/$site_id/slug/$page_slug/links/to/missing");
        return self::readJson($resp);
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

            return $this->parseUser($resp, $detail);
        }, "No user found with ID $id");
    }

    public function getUserBySlug(string $slug, string $detail = 'identity'): ?User
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

    private function parseCategory($resp): Category
    {
        $category = self::readJson($resp);
        self::convertDateProperties($category, ['createdAt', 'updatedAt']);
        return new Category($category);
    }

    private function parsePage($resp): Page
    {
        $page = self::readJson($resp);

        self::convertDateProperties($page, [
            'pageCreatedAt',
            'pageUpdatedAt',
            'pageDeletedAt',
            'revisionCreatedAt',
            'compiledAt',
        ]);

        return new Page($page);
    }

    private function parseUser($resp, string $detail): User
    {
        $user = self::readJson($resp);

        self::convertDateProperties($user, [
            'since',
            'lastActive',
            'birthday',
            'emailVerifiedAt',
            'createdAt',
            'updatedAt',
            'deletedAt',
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
        $route = $full ? 'version/full' : 'version';
        return (string) $this->client->get($route)->getBody();
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

    /**
     * PHP booleans are technically ints...
     *
     * This takes a boolean value and makes an explicit string 'true' or 'false'.
     * @param bool $value
     * @return string
     */
    private static function booleanValue(bool $value): string
    {
        return $value ? 'true' : 'false';
    }
}
