<?php
declare(strict_types=1);

namespace Wikijump\Services\Deepwell\Models;

use Carbon\Carbon;
use Ds\Set;
use Wikijump\Services\Deepwell\DeepwellService;

class Page extends DeepwellModel
{
    // Fields and constructor
    public int $page_id;
    public Carbon $page_created_at;
    public ?Carbon $page_updated_at;
    public ?Carbon $page_deleted_at;
    public int $page_revision_count;
    public int $site_id;
    public int $page_category_id;
    public string $page_category_slug;
    public ?int $discussion_thread_id;
    public int $revision_id;
    public Carbon $revision_created_at;
    public int $revision_number;
    public int $revision_user_id;
    public ?string $wikitext;
    public ?string $compiled_html;
    public Carbon $compiled_at;
    public string $compiled_generator;
    public string $revision_comments;
    public array $hidden_fields;
    public string $title;
    public ?string $alt_title;
    public string $slug;
    public Set $tags;
    public object $metadata; // TODO change type when we figure out the metadata structure

    public function __construct(object $raw_page)
    {
        $this->page_id = $raw_page->pageId;
        $this->page_created_at = $raw_page->pageCreatedAt;
        $this->page_updated_at = $raw_page->pageUpdatedAt;
        $this->page_deleted_at = $raw_page->pageDeletedAt;
        $this->page_revision_count = $raw_page->pageRevisionCount;
        $this->site_id = $raw_page->siteId;
        $this->page_category_id = $raw_page->pageCategoryId;
        $this->page_category_slug = $raw_page->pageCategorySlug;
        $this->discussion_thread_id = $raw_page->discussionThreadId;
        $this->revision_id = $raw_page->revisionId;
        $this->revision_created_at = $raw_page->revisionCreatedAt;
        $this->revision_number = $raw_page->revisionNumber;
        $this->revision_user_id = $raw_page->revisionUserId;
        $this->wikitext = $raw_page->wikitext;
        $this->compiled_html = $raw_page->compiledHtml;
        $this->compiled_at = $raw_page->compiledAt;
        $this->compiled_generator = $raw_page->compiledGenerator;
        $this->revision_comments = $raw_page->revisionComments;
        $this->hidden_fields = $raw_page->hiddenFields;
        $this->title = $raw_page->title;
        $this->alt_title = $raw_page->altTitle;
        $this->slug = $raw_page->slug;
        $this->tags = new Set($raw_page->tags);
        $this->metadata = $raw_page->metadata;
    }

    // Fetch methods
    public static function findSlug($site_id, string $page_slug): ?Page
    {
        // NOTE: We cast arbitrary input to int, since Wikidot uses strings for IDs in most places
        return DeepwellService::getInstance()->getPageBySlug(
            intval($site_id),
            $page_slug,
        );
    }

    public static function findId($site_id, int $page_id): ?Page
    {
        // NOTE: We cast arbitrary input to int, since Wikidot uses strings for IDs in most places
        return DeepwellService::getInstance()->getPageById(intval($site_id), $page_id);
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
     * @param string|int $page_id The ID of the Page to find
     * @return ?Page The page if found, or null
     */
    public static function findIdOnly($page_id): ?Page
    {
        // NOTE: We cast arbitrary input to int, since Wikidot uses strings for IDs in most places
        return DeepwellService::getInstance()->getPageByIdOnly($page_id);
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
}
