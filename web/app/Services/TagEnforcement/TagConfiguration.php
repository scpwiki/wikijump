<?php
declare(strict_types=1);

namespace Wikijump\Services\TagEnforcement;

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
     *     ?'required_if_valid' => bool,
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
     *     ?'required_if_valid' => bool,
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
            if (isset($tag['properties']['role_ids'])) {
                $tag['properties']['role_ids'] = new Set($tag['properties']['role_ids']);
            }
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

    /**
     * Validates whether the added and removed tags all exist and can be changed in this context.
     *
     * Result schema:
     * [
     *   'tag-name' => ['invalid' | 'undefined' | 'role' | 'date'...],
     * ]
     *
     * @param Set $added_tags Which tags were added
     * @param Set $removed_tags Which tags were removed
     * @param Set $role_ids The roles that the current user performing the tag action has
     * @param Carbon $date
     * @return array The result of the determination
     */
    public function validateTags(
        Set $added_tags,
        Set $removed_tags,
        Set $role_ids,
        Carbon $date
    ): array {
        $result = [];

        // If empty, it means don't validate on tag existence
        if (empty($this->tags)) {
            return $result;
        }

        // Check all added and removed tags
        foreach ($added_tags as $tag) {
            $this->checkCanChangeTag($tag, $role_ids, $date, $result);
        }

        foreach ($removed_tags as $tag) {
            $this->checkCanChangeTag($tag, $role_ids, $date, $result);
        }

        return $result;
    }

    /**
     * Validates whether all tag conditions passed for this set of new tags.
     *
     * Result schema:
     * [
     *   'tags' => [
     *     'tag-name' => [
     *       For each condition:
     *       [
     *         'valid' => bool,
     *         'passed' => int,
     *         'threshold' => int,
     *       ],
     *       ...
     *     ],
     *   ],
     *   'tag_groups' => [
     *     'tag-group-name' => [
     *       For each condition:
     *       [
     *         'valid' => bool,
     *         'passed' => int,
     *         'threshold' => int,
     *       ],
     *       ...
     *     ],
     *   ],
     * ]
     *
     * @param Set $tags The tags being proposed
     * @return array The result of the determination
     */
    public function validateConditions(Set $tags): array
    {
        $result = [
            'tags' => [],
            'tag_groups' => [],
        ];

        // Check tag condition lists
        foreach ($this->tags as $tag => $tag_data) {
            $present = $tags->contains($tag);
            $required_if_valid = $tag_data['required_if_valid'] ?? false;

            if ($tags->contains($tag) || $required_if_valid) {
                $results = $this->gatherConditionListResults($tags, $tag_data, $present, $required_if_valid);

                if (!empty($results)) {
                    $result['tags'][$tag] = $results;
                }
            }
        }

        // Check tag group condition lists
        foreach ($this->tag_groups as $tag_group => $tag_group_data) {
            $tag_group_empty = $this->tagGroupPresent($tag_group, $tags)->isEmpty();
            $required_if_valid = $tag_group_data['required_if_valid'] ?? false;

            if (!$tag_group_empty || $required_if_valid) {
                $results = $this->gatherConditionListResults($tags, $tag_group_data, !$tag_group_empty, $required_if_valid);

                if (!empty($results)) {
                    $result['tag_groups'][$tag_group] = $results;
                }
            }
        }

        // All condition lists passed
        return $result;
    }

    // Tag helpers
    private function gatherConditionListResults(Set $tags, array $data, bool $present, bool $required_if_valid): array
    {
        $results = [];
        $valid = true;

        // Process all condition lists
        foreach ($data['condition_lists'] as $condition_list_data) {
            $condition_list = new TagConditionList($this, $condition_list_data);
            $result = $condition_list->validate($tags, $present);
            $results[] = $result;

            if (!$result['valid']) {
                $valid = false;
            }
        }

        // Required to be present
        if ($valid && $required_if_valid) {
            $results[] = [
                'valid' => $present,
                'required' => true,
                'type' => 'required_if_valid',
            ];
        }

        return $results;
    }

    private function checkCanChangeTag(
        string $tag,
        Set $role_ids,
        Carbon $date,
        array &$result
    ): void {
        $reasons = [];

        if ($tag === '') {
            // Special case, tags can't be empty strings
            $reasons[] = 'invalid';
            $result[$tag] = $reasons;
            return;
        }

        $tag_data = $this->tags[$tag];
        if ($tag_data === null) {
            // No tag entry, not a valid tag
            $reasons[] = 'undefined';
            $result[$tag] = $reasons;
            return;
        }

        // Check role constraint, if present
        $allowed_role_ids = $tag_data['properties']['role_ids'];
        if ($allowed_role_ids !== null) {
            if ($allowed_role_ids->intersect($role_ids)->isEmpty()) {
                // No roles in common, not allowed to apply
                $reasons[] = 'role';
            }
        }

        // Check date constraint, if present
        $date_bound = $tag_data['properties']['date_bound'];
        if ($date_bound !== null) {
            [$start_date, $end_date] = $date_bound;
            if (!$date->between($start_date, $end_date)) {
                $reasons[] = 'date';
            }
        }

        // If there are reasons for rejection, add them
        if (!empty($reasons)) {
            $result[$tag] = $reasons;
        }
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
