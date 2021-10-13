<?php
declare(strict_types=1);

namespace Wikijump\Services\TagEngine;

/**
 * The Tag Engine receives the information of the site's tagging configuration and processes new page tags.
 */

 final class TagEngine {

    private function __construct() {} // To prevent instantiation.

    public static function validate(object $configuration, array $tags): TagDecision {

        if ($configuration->allowedTags !== []) {
            /* Check tags here. */
        }

        return new TagDecision($valid, $forbiddenTags);
    }

 }
