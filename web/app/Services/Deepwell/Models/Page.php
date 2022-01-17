<?php
declare(strict_types=1);

namespace Wikijump\Services\Deepwell\Models;

use Carbon\Carbon;
use Wikijump\Services\Deepwell\DeepwellService;

class Page extends DeepwellModel
{
    // Fields and constructor
    private int $id;
    private Carbon $created_at;
    private ?Carbon $updated_at;
    private ?Carbon $deleted_at;
    private int $site_id;
    private int $category_id;
    private string $slug;
    private int $discussion_thread_id;

    public function __construct(object $raw_page)
    {
        $this->id = $raw_page->page_id;
        $this->created_at = $raw_page->created_at;
        $this->updated_at = $raw_page->updated_at;
        $this->deleted_at = $raw_page->deleted_at;
        $this->site_id = $raw_page->site_id;
        $this->category_id = $raw_page->page_category_id;
        $this->slug = $raw_page->slug;
        $this->discussion_thread_id = $raw_page->discussion_thread_id;
    }

    // Fetch methods
    public static function findId(int $site_id, int $page_id): ?Page
    {
        return DeepwellService::getInstance()->getPageById($site_id, $page_id);
    }

    public static function findSlug(int $site_id, string $page_slug): ?User
    {
        return DeepwellService::getInstance()->getPageBySlug($site_id, $page_slug);
    }

    /**
     * Returns the page with the given ID.
     *
     * This method is *not* preferred over findId(),
     * because the site ID check helps ensure security.
     * The idea is to avoid an attacker, if they know the
     * ID of a hidden page, smuggles it into a request and
     * has the backend retrieve it for them.
     *
     * @param int $page_id The ID of the Page to find
     * @return ?Page The page if found, or null
     */
    public static function findIdOnly(int $page_id): ?Page
    {
        throw new Exception('DEEPWELL getPageByIdOnly not implemented yet');
    }

    // Getters
    public function id(): int
    {
        return $this->id;
    }

    public function exists(): bool
    {
        return $this->deleted_at === null;
    }

    public function siteId(): int
    {
        return $this->site_id;
    }

    public function categoryId(): int
    {
        return $this->category_id;
    }

    public function slug(): string
    {
        return $this->slug;
    }

    // Instance methods
    // TODO
}
