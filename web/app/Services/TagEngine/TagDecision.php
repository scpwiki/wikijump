<?php
declare(strict_types=1);

namespace Wikijump\Services\TagEngine;

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

    public function __construct(bool $valid)
    {
        $this->valid = $valid;
    }
}
