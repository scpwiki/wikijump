<?php
declare(strict_types=1);

namespace Wikijump\Services\Deepwell\Models;

use Carbon\Carbon;
use Wikijump\Services\Deepwell\DeepwellService;

class Category
{
    // Fields and constructor
    public int $category_id;
    public Carbon $created_at;
    public ?Carbon $updated_at;
    public int $site_id;
    public string $slug;

    public function __construct(object $raw_category)
    {
        $this->category_id = $raw_category->categoryId;
        $this->created_at = $raw_category->createdAt;
        $this->updated_at = $raw_category->updatedAt;
        $this->site_id = $raw_category->siteId;
        $this->slug = $raw_category->slug;
    }

    // Fetch methods
    public static function findSlug($site_id, string $category_slug): ?Category
    {
        // NOTE: We cast arbitrary input to int, since Wikidot uses strings for IDs in most places
        return DeepwellService::getInstance()->getCategoryBySlug(
            intval($site_id),
            $category_slug,
        );
    }

    public static function findId($site_id, int $category_id): ?Category
    {
        // NOTE: We cast arbitrary input to int, since Wikidot uses strings for IDs in most places
        return DeepwellService::getInstance()->getCategoryById(
            intval($site_id),
            $category_id,
        );
    }

    public static function findIdOnly($category_id): ?Category
    {
        // NOTE: We cast arbitrary input to int, since Wikidot uses strings for IDs in most places
        return DeepwellService::getInstance()->getCategoryByIdOnly(intval($category_id));
    }

    public static function findAll($site_id): array
    {
        // NOTE: We cast arbitrary input to int, since Wikidot uses strings for IDs in most places
        return DeepwellService::getInstance()->getCategories(intval($site_id));
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
