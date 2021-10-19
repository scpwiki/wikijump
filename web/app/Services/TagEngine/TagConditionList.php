<?php
declare(strict_types=1);

namespace Wikijump\Services\TagEngine;

use Ds\Set;
use Exception;

class TagConditionList
{
    private TagConfiguration $config;

    /**
     * @var array The definition for a set of conditions to be verified
     *
     * Schema: [
     *   'requires' => 'all-of' | 'none-of' | 'custom',
     *    if 'custom' only:
     *   'compare' => 'gte' | 'gt' | 'lte' | 'lt' | 'eq' | 'neq',
     *   'value' => integer,
     *
     *   'conditions' => [
     *     one or more:
     *     [
     *       'type' => 'tag-is-present' | 'tag-is-absent' | 'tag-group-is-fully-present' | 'tag-group-is-fully-absent' | 'tag-group-custom',
     *       if 'tag-group-custom' only:
     *       'compare' => 'gte' | 'gt' | 'lte' | 'lt' | 'eq' | 'neq',
     *       'value' => integer,
     *
     *       'name' => 'tag or tag group name',
     *     ],
     *   ],
     * ]
     */
    private array $condition_list;

    public function __construct(TagConfiguration $config, array $condition_list)
    {
        $this->config = $config;
        $this->condition_list = $condition_list;
    }

    public function validate(Set $tags): bool
    {
        $total = count($this->condition_list['conditions']);
        $passed = 0;

        foreach ($this->condition_list['conditions'] as $condition) {
            if ($this->validateCondition($condition, $tags)) {
                $passed++;
            }
        }

        $type = $this->condition_list['requires'];
        switch ($type) {
            case 'all-of':
                return $passed === $total;
            case 'none-of':
                return $passed === 0;
            case 'custom':
                return $this->checkCustomRequirement($this->condition_list, $passed);
            default:
                throw new Exception("Unknown requirement type: $type");
        }
    }

    private function validateCondition(array $condition, Set $tags): bool
    {
        $type = $condition['type'];
        $name = $condition['name'];
        switch ($type) {
            case 'tag-is-present':
                return $tags->contains($name);
            case 'tag-is-absent':
                return !$tags->contains($name);
            case 'tag-group-is-fully-present':
                return $this->config->tagGroupFullyPresent($name, $tags);
            case 'tag-group-is-fully-absent':
                return $this->config->tagGroupFullyAbsent($name, $tags);
            case 'tag-group-custom':
                $present = count($this->config->tagGroupPresent($name, $tags));
                return $this->checkCustomRequirement($condition, $present);
            default:
                throw new Exception("Unknown condition type: $type");
        }
    }

    private function checkCustomRequirement(array $requirement, int $x): bool
    {
        $type = $requirement['compare'];
        $y = $requirement['value'];

        switch ($type) {
            case 'gte':
                return $x >= $y;
            case 'gt':
                return $x > $y;
            case 'lte':
                return $x <= $y;
            case 'lt':
                return $x < $y;
            case 'eq':
                return $x === $y;
            case 'neq':
                return $x !== $y;
            default:
                throw new Exception("Unknown comparison type: $type");
        }
    }
}
