<?php
declare(strict_types=1);

namespace Wikijump\Services\TagEngine;

use Ds\Set;

/**
 * The Tag Engine receives the information of the site's tagging configuration and processes new page tags.
 */

final class TagEngine
{
    private function __construct()
    {
    }

    /**
     * Verifies whether the given tags are valid or not under the passed tag configuration.
     *
     * @param TagConfiguration $config The configuration to check the tags against
     * @param Set $previous_tags The previous tags (if any) associated with this page
     * @param Set $current_tags The tags being proposed for this page
     * @return TagDecision The outcome of validation
     */
    public static function validate(
        TagConfiguration $config,
        Set $previous_tags,
        Set $current_tags
    ): TagDecision {
        $added_tags = $current_tags.difference($previous_tags);
        $removed_tags = $previous_tags.difference($current_tags);

        $valid = true; // Whether or not the tags being saved are valid according to the site's configuration.
        $forbidden_tags = null; // What tags present in the current tags are not allowed on the site.

        /* Return validator's decision along with relevant information if the tags are invalid. */
        return new TagDecision($valid, $forbidden_tags);
    }
}
