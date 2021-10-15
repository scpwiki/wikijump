<?php
declare(strict_types=1);

namespace Wikijump\Services\TagEngine;

use Ds/Set;

/**
 * The Tag Engine receives the information of the site's tagging configuration and processes new page tags.
 */

 final class TagEngine {

    private function __construct() {} // To prevent instantiation.

    public static function validate(object $configuration, array $current_tags, array $previous_tags): TagDecision {

        $current_tags   = new Set($current_tags); // Tags being saved to the page.
        $previous_tags  = new Set($previous_tags); // Tags previously saved to the page.
        $allowed_tags   = new Set($configuration->allowed_tags); // What tags are allowed on the site.

        $valid          = true; // Whether or not the tags being saved are valid according to the site's configuration.
        $forbidden_tags = null; // What tags present in the current tags are not allowed on the site.


        /* Check the tags against the tag whitelist. */
        if (!$allowed_tags->isEmpty()) {
            $forbidden_tags = $current_tags->diff($allowed_tags);
            if(!$forbidden_tags->isEmpty()) {
                $valid = false;
            }
        }

        /* Return validator's decision along with relevant information if the tags are invalid. */
        return new TagDecision($valid, $forbidden_tags);
    }

 }
