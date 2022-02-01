<?php
declare(strict_types=1);

namespace Wikijump\Services\Deepwell\Models;

use Carbon\Carbon;
use Wikijump\Services\Deepwell\DeepwellService;

class Page extends DeepwellModel
{
    // Fields and constructor
    public int $page_id;
    public Carbon $created_at;
    public ?Carbon $updated_at;
    public ?Carbon $deleted_at;
    public int $site_id;
    public int $category_id;
    public string $slug;
    public int $discussion_thread_id;

    public function __construct(object $raw_page)
    {
        $this->page_id = $raw_page->pageId;
        $this->created_at = $raw_page->createdAt;
        $this->updated_at = $raw_page->updatedAt;
        $this->deleted_at = $raw_page->deletedAt;
        $this->site_id = $raw_page->siteId;
        $this->category_id = $raw_page->pageCategoryId;
        $this->slug = $raw_page->slug;
        $this->discussion_thread_id = $raw_page->discussionThreadId;
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
        return $this->page_id;
    }

    public function exists(): bool
    {
        return $this->deleted_at === null;
    }

    // Instance methods
    public function getLastRevision(): PageRevision
    {
        return DeepwellService::getInstance()->getLatestPageRevision(
            $this->site_id,
            $this->page_id,
        );
    }

    public function getRevision(int $revision_number): ?PageRevision
    {
        return DeepwellService::getInstance()->getPageRevision(
            $this->site_id,
            $this->page_id,
            $revision_number,
        );
    }
}
