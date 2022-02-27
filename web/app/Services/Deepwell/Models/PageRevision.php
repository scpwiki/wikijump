<?php
declare(strict_types=1);

namespace Wikijump\Services\Deepwell\Models;

use Carbon\Carbon;
use Wikijump\Services\Deepwell\DeepwellService;

class PageRevision extends DeepwellModel
{
    // Fields and constructor
    public int $revision_id;
    public Carbon $created_at;
    public int $revision_number;
    public int $page_id;
    public int $site_id;
    public int $user_id;
    public string $wikitext_hash;
    public string $wikitext;
    public string $compiled_html_hash;
    public string $compiled_html;
    public Carbon $compiled_at;
    public string $compiled_generator;
    public string $comments;
    public array $hidden;
    public string $title;
    public ?string $alt_title;
    public string $slug;
    public array $tags;
    public array $metadata;

    public function __construct(object $raw_revision)
    {
        $this->revision_id = $raw_revision->revisionId;
        $this->created_at = $raw_revision->createdAt;
        $this->revision_number = $raw_revision->revisionNumber;
        $this->page_id = $raw_revision->pageId;
        $this->site_id = $raw_revision->siteId;
        $this->user_id = $raw_revision->userId;
        $this->wikitext_hash = self::byteArrayToHex($raw_revision->wikitextHash);
        $this->compiled_html_hash = self::byteArrayToHex($raw_revision->compiledHash);
        $this->compiled_at = $raw_revision->compiledAt;
        $this->compiled_generator = $raw_revision->compiledGenerator;
        $this->comments = $raw_revision->comments;
        $this->hidden = $raw_revision->hidden;
        $this->title = $raw_revision->title;
        $this->alt_title = $raw_revision->altTitle;
        $this->slug = $raw_revision->slug;
        $this->tags = $raw_revision->tags;
        $this->metadata = $raw_revision->metadata;

        // Fetch full texts
        $this->wikitext = DeepwellService::getInstance()->getText($this->wikitext_hash);
        $this->compiled_html = DeepwellService::getInstance()->getText(
            $this->compiled_html_hash,
        );
    }

    // Helper methods
    private static function byteArrayToHex(array $bytes): string
    {
        return bin2hex(join(array_map('chr', $bytes)));
    }

    // Fetch methods
    public static function findLatest(int $site_id, int $page_id): ?PageRevision
    {
        return DeepwellService::getInstance()->getLatestPageRevision($site_id, $page_id);
    }

    public static function findNumber(
        int $site_id,
        int $page_id,
        int $revision_number
    ): ?PageRevision {
        return DeepwellService::getInstance()->getPageRevision(
            $site_id,
            $page_id,
            $revision_number,
        );
    }

    // Getters
    public function id(): int
    {
        return $this->revision_id;
    }

    // Instance methods
    public function editHidden(int $user_id, array $hidden): void
    {
        $this->hidden = $hidden;
        DeepwellService::getInstance()->setPageRevision(
            $this->site_id,
            $this->page_id,
            $this->revision_number,
            $user_id,
            $hidden,
        );
    }
}
