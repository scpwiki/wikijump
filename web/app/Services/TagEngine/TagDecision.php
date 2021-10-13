<?php
declare(strict_types=1);

namespace Wikijump\Services\TagEngine;

/**
 * The TagDecision Class stores the validation information for the TagEngine to return, including whether the inputted tags are valid, and if not valid, the reason for the invalidity.
 */

 class TagDecision {
     public bool  $valid;
     public array $forbiddenTags;

     public function __construct(
         bool $valid,
         array $forbiddenTags
     ) 
     {
         $this->valid = $valid;
         $this->forbiddenTags = $forbiddenTags;
     }
 }
