<?php
declare(strict_types=1);

namespace Wikijump\Services\TagEngine;

use Carbon\Carbon;
use Ds\Set;

class TagConfiguration
{
    /**
     * @var array $tags The rules and definitions concerning tags.
     *
     * Schema: [
     *   'tag_name' => [
     *     'properties' => [
     *        ?'role_ids' => [list of role IDs that can apply this tag],
     *        ?'date_bound' => [start_date, end_date],
     *      ],
     *     'condition_lists' => [
     *       zero or more:
     *       array matching TagConditionList
     *     ],
     *   ],
     * ]
     */
    private array $tags;

    /**
     * @var array $tag_groups The rules and definitions concerning tags.
     *
     * Schema: [
     *   'group_name' => [
     *     'members' => [list of tag names],
     *     'condition_lists' => [
     *       zero or more:
     *       array matching TagConditionList
     *     ],
     *   ],
     * ]
     */
    private array $tag_groups;

    // Creation and serialization
    public function __construct(array $data)
    {
        $this->tags = $data['tags'] ?? [];
        $this->tag_groups = $data['tag_groups'] ?? [];

        // Convert $tags
        foreach ($this->tags as &$tag) {
            $tag['properties']['role_ids'] = new Set($tag['properties']['role_ids']);
        }

        // Convert $tag_groups
        foreach ($this->tag_groups as &$tag_group) {
            $tag_group['members'] = new Set($tag_group['members']);
        }
    }

    /**
     * Exposes a view of this class that is suitable for serializing into JSON.
     * @return array
     */
    public function toJson(): array
    {
        return [
            'tags' => $this->tags,
            'tag_groups' => $this->tag_groups,
        ];
    }

    // Validation
    public function validateTags(
        Set $added_tags,
        Set $removed_tags,
        Set $role_ids,
        Carbon $date
    ): bool {
        // If empty, it means don't validate on tag existence
        if (empty($this->tags)) {
            return true;
        }

        // Check all added and removed tags
        foreach ($added_tags as $tag) {
            if (!$this->canChangeTag($tag, $date, $role_ids)) {
                return false;
            }
        }

        foreach ($removed_tags as $tag) {
            if (!$this->canChangeTag($tag, $date, $role_ids)) {
                return false;
            }
        }

        return true;
    }

    public function validateConditions(Set $tags): bool
    {
        // Check tag condition lists
        foreach ($this->tags as $tag => $data) {
            if ($tags->contains($tag)) {
                foreach ($data['condition_lists'] as $condition_list_data) {
                    $condition_list = new TagConditionList($condition_list_data);
                    if (!$condition_list->validate($tags)) {
                        return false;
                    }
                }
            }
        }

        // Check tag group condition lists
        foreach ($this->tag_groups as $tag_group => $data) {
            if (!$this->tagGroupPresent($tag_group, $tags)->isEmpty()) {
                foreach ($data['condition_lists'] as $condition_list_data) {
                    $condition_list = new TagConditionList($condition_list_data);
                    if (!$condition_list->validate($tags)) {
                        return false;
                    }
                }
            }
        }

        // All condition lists passed
        return true;
    }

    // Tag helpers
    private function canChangeTag(string $tag, Set $role_ids, Carbon $date): bool
    {
        $tag_data = $this->tags[$tag];
        if ($tag_data === null) {
            // No tag entry, not a valid tag
            return false;
        }

        // Check role constraint, if present
        $allowed_role_ids = $tag_data['properties']['role_ids'];
        if ($allowed_role_ids !== null) {
            if ($allowed_role_ids->intersect($role_ids)->isEmpty()) {
                // No roles in common, not allowed to apply
                return false;
            }
        }

        // Check date constraint, if present
        $date_bound = $tag_data['properties']['date_bound'];
        if ($date_bound !== null) {
            [$start_date, $end_date] = $date_bound;
            if (!$date->between($start_date, $end_date)) {
                return false;
            }
        }

        // All constraints passed
        return true;
    }

    // Tag group helpers
    public function tagGroupMembers(string $name): Set
    {
        return $this->tag_groups[$name]['members'];
    }

    public function tagGroupPresent(string $name, Set $tags): Set
    {
        return $this->tagGroupMembers($name)->intersect($tags);
    }

    public function tagGroupFullyPresent(string $name, Set $tags): bool
    {
        return $this->tagGroupMembers($name)
            ->diff($tags)
            ->isEmpty();
    }

    public function tagGroupFullyAbsent(string $name, Set $tags): bool
    {
        return $this->tagGroupPresent($name, $tags)->isEmpty();
    }
}
