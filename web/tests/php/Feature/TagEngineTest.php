<?php
declare(strict_types=1);

namespace Tests\Feature;

use Carbon\Carbon;
use Ds\Set;
use Tests\TestCase;
use Wikijump\Services\TagEnforcement\TagConfiguration;
use Wikijump\Services\TagEnforcement\TagEngine;

class TagEngineTest extends TestCase
{
    private function checkDecision(
        TagConfiguration $config,
        array $previous_tags,
        array $current_tags,
        array $role_ids,
        array $expected_decision
    ): void {
        $actual_decision = TagEngine::validate(
            $config,
            new Set($previous_tags),
            new Set($current_tags),
            new Set($role_ids),
        );

        $this->assertEquals(
            $expected_decision,
            $actual_decision->toJson(),
            "Actual decision array doesn't match expected",
        );
    }

    /**
     * Tests the TagEngine on a default configuration.
     *
     * Because this configuration doesn't actually enforce anything,
     * all the test cases should result in a valid outcome.
     *
     * @return void
     */
    public function testDefaultConfiguration(): void
    {
        $config = new TagConfiguration([]);

        $this->checkDecision(
            $config,
            ['a', 'b'],
            ['a', 'b', 'c'],
            [],
            [
                'valid' => true,
                'invalid_tags' => [],
                'tag_conditions' => [],
                'tag_group_conditions' => [],
            ],
        );

        $this->checkDecision(
            $config,
            ['banana', 'cherry'],
            ['apple', 'banana'],
            [],
            [
                'valid' => true,
                'invalid_tags' => [],
                'tag_conditions' => [],
                'tag_group_conditions' => [],
            ],
        );
    }

    /**
     * Tests the TagEngine on a simple configuration that verifies if tags exist.
     *
     * This only checks if the tags come from a selected whitelist, and no other properties.
     *
     * @return void
     */
    public function testAllowedTags(): void
    {
        $tag_exists = [
            'properties' => [],
            'condition_lists' => [],
        ];

        $config = new TagConfiguration([
            'tags' => [
                'apple' => $tag_exists,
                'banana' => $tag_exists,
                'cherry' => $tag_exists,
            ],
        ]);

        $this->checkDecision(
            $config,
            ['apple', 'banana'],
            ['apple', 'cherry'],
            [],
            [
                'valid' => true,
                'invalid_tags' => [],
                'tag_conditions' => [],
                'tag_group_conditions' => [],
            ],
        );

        $this->checkDecision(
            $config,
            [],
            ['apple', 'banana', 'cherry'],
            [],
            [
                'valid' => true,
                'invalid_tags' => [],
                'tag_conditions' => [],
                'tag_group_conditions' => [],
            ],
        );

        $this->checkDecision(
            $config,
            ['apple', 'banana', 'cherry'],
            [],
            [],
            [
                'valid' => true,
                'invalid_tags' => [],
                'tag_conditions' => [],
                'tag_group_conditions' => [],
            ],
        );

        $this->checkDecision(
            $config,
            ['apple', 'banana'],
            ['apple', 'banana', 'durian'],
            [],
            [
                'valid' => false,
                'invalid_tags' => [
                    'durian' => ['undefined'],
                ],
                'tag_conditions' => [],
                'tag_group_conditions' => [],
            ],
        );

        $this->checkDecision(
            $config,
            ['zebra', 'apple'],
            ['zebra', 'apple', 'banana'],
            [],
            [
                'valid' => true,
                'invalid_tags' => [],
                'tag_conditions' => [],
                'tag_group_conditions' => [],
            ],
        );
    }

    /**
     * Tests the TagEngine where some tags are restricted to certain roles or dates.
     *
     * This checks if tag properties are valid for each applied.
     *
     * @return void
     */
    public function testRestrictedTags(): void
    {
        $config = new TagConfiguration([
            'tags' => [
                'chocolate' => [
                    'properties' => [],
                    'condition_lists' => [],
                ],
                'vanilla' => [
                    'properties' => [],
                    'condition_lists' => [],
                ],
                'old-contest' => [
                    'properties' => [
                        'date_bound' => [
                            new Carbon('2008-01-01'),
                            new Carbon('2008-03-01'),
                        ],
                    ],
                    'condition_lists' => [],
                ],
                'future-contest' => [
                    'properties' => [
                        'date_bound' => [
                            new Carbon('2500-01-01'),
                            new Carbon('2500-03-01'),
                        ],
                    ],
                    'condition_lists' => [],
                ],
                'staff-process' => [
                    'properties' => [
                        'role_ids' => [20, 30],
                    ],
                    'condition_lists' => [],
                ],
            ],
        ]);

        // Not changing restricted tags is fine
        $this->checkDecision(
            $config,
            ['old-contest', 'chocolate'],
            ['old-contest', 'vanilla'],
            [10],
            [
                'valid' => true,
                'invalid_tags' => [],
                'tag_conditions' => [],
                'tag_group_conditions' => [],
            ],
        );

        $this->checkDecision(
            $config,
            ['old-contest', 'chocolate'],
            ['old-contest', 'vanilla'],
            [10],
            [
                'valid' => true,
                'invalid_tags' => [],
                'tag_conditions' => [],
                'tag_group_conditions' => [],
            ],
        );

        // Changing restricted tags fails
        $this->checkDecision(
            $config,
            [],
            ['old-contest', 'vanilla'],
            [10],
            [
                'valid' => false,
                'invalid_tags' => [
                    'old-contest' => ['date'],
                ],
                'tag_conditions' => [],
                'tag_group_conditions' => [],
            ],
        );

        $this->checkDecision(
            $config,
            [],
            ['staff-process', 'vanilla'],
            [10],
            [
                'valid' => false,
                'invalid_tags' => [
                    'staff-process' => ['role'],
                ],
                'tag_conditions' => [],
                'tag_group_conditions' => [],
            ],
        );

        // ...unless you have the correct role
        $this->checkDecision(
            $config,
            [],
            ['staff-process', 'vanilla'],
            [10, 20],
            [
                'valid' => true,
                'invalid_tags' => [],
                'tag_conditions' => [],
                'tag_group_conditions' => [],
            ],
        );

        // But even with the role you can't change old tags
        $this->checkDecision(
            $config,
            [],
            ['old-contest', 'vanilla'],
            [10, 20],
            [
                'valid' => false,
                'invalid_tags' => [
                    'old-contest' => ['date'],
                ],
                'tag_conditions' => [],
                'tag_group_conditions' => [],
            ],
        );

        // This also works for future dates
        $this->checkDecision(
            $config,
            [],
            ['future-contest', 'vanilla'],
            [10, 20],
            [
                'valid' => false,
                'invalid_tags' => [
                    'future-contest' => ['date'],
                ],
                'tag_conditions' => [],
                'tag_group_conditions' => [],
            ],
        );
    }
}
