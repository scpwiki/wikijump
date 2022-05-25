<?php
declare(strict_types=1);

namespace Wikijump\Services\Deepwell\Models;

use Carbon\Carbon;
use Ds\Set;
use Wikijump\Services\Deepwell\DeepwellService;

class File extends DeepwellModel
{
    // Fields and constructor
    public string $file_id;
    public Carbon $created_at;
    public ?Carbon $updated_at;
    public ?Carbon $deleted_at;
    public string $name;
    public string $s3_hash;
    public int $user_id;
    public int $page_id;
    public int $site_hint;
    public string $mime_hint;
    public array $licensing;

    public function __construct(object $raw_file)
    {
        $this->$file_id = $raw_file->fileId;
        $this->$created_at = $raw_file->createdAt;
        $this->$updated_at = $raw_file->updatedAt;
        $this->$deleted_at = $raw_file->deletedAt;
        $this->$name = $raw_file->name;
        $this->$s3_hash = $raw_file->s3_hash;
        $this->$user_id = $raw_file->user_id;
        $this->$page_id = $raw_file->page_id;
        $this->$site_hint = $raw_file->site_hint;
        $this->$mime_hint = $raw_file->mime_hint;
        $this->$licensing = $raw_file->licensing;
    }

    // Fetch methods
    public function findName($site_id, $page_id, string $filename): ?File
    {
        // TODO
        return null;
    }

    public function findId(string $file_id): ?File
    {
        // TODO
        return null;
    }

    public function findFromPage($page_id): array
    {
        // TODO
        return [];
    }
}
