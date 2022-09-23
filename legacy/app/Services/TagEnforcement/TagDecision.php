<?php
declare(strict_types=1);

namespace Wikijump\Services\TagEnforcement;

/**
 * The TagDecision Class stores the validation information for the TagEngine to return, including whether the inputted tags are valid, and if not valid, the reason for the invalidity.
 */

/**
 * A description of a decision from the TagEngine.
 *
 * It has a singular field $valid to describe whether the tags overall are acceptable,
 * and if not, has a series of fields describing the issues in detail.
 */
class TagDecision
{
    public bool $valid;
    public array $invalid_tags;
    public array $tag_conditions;
    public array $tag_group_conditions;

    public function __construct(
        bool $valid,
        array $invalid_tags,
        array $tag_conditions,
        array $tag_group_conditions
    ) {
        $this->valid = $valid;
        $this->invalid_tags = $invalid_tags;
        $this->tag_conditions = $tag_conditions;
        $this->tag_group_conditions = $tag_group_conditions;
    }

    public function toJson(): array
    {
        return [
            'valid' => $this->valid,
            'invalid_tags' => $this->invalid_tags,
            'tag_conditions' => $this->tag_conditions,
            'tag_group_conditions' => $this->tag_group_conditions,
        ];
    }
}
