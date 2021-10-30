<?php
declare(strict_types=1);

namespace Tests\Feature;

use Carbon\Carbon;
use Ds\Set;
use Tests\TestCase;
use Wikijump\Services\TagEnforcement\TagConfiguration;
use Wikijump\Services\TagEnforcement\TagEngine;

class TagEnforcementTest extends TestCase
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
            [],
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

    /**
     * Tests the TagEngine's capability with simple "requires" and "conflicts" conditions.
     *
     * @return void
     */
    public function testSimpleDependency(): void
    {
        $config = new TagConfiguration([
            'tags' => [
                // Parent tags
                'fruit' => [
                    'properties' => [],
                    'condition_lists' => [
                        [
                            'requires' => 'all-of',
                            'conditions' => [
                                [
                                    'type' => 'tag-is-absent',
                                    'name' => 'vegetable',
                                ],
                            ],
                        ],
                    ],
                ],
                'vegetable' => [
                    'properties' => [],
                    'condition_lists' => [
                        [
                            'requires' => 'all-of',
                            'conditions' => [
                                [
                                    'type' => 'tag-is-absent',
                                    'name' => 'fruit',
                                ],
                            ],
                        ],
                    ],
                ],

                // Child tags
                'apple' => [
                    'properties' => [],
                    'condition_lists' => [
                        [
                            'requires' => 'all-of',
                            'conditions' => [
                                [
                                    'type' => 'tag-is-present',
                                    'name' => 'fruit',
                                ],
                            ],
                        ],
                    ],
                ],
                'banana' => [
                    'properties' => [],
                    'condition_lists' => [
                        [
                            'requires' => 'all-of',
                            'conditions' => [
                                [
                                    'type' => 'tag-is-present',
                                    'name' => 'fruit',
                                ],
                            ],
                        ],
                    ],
                ],

                'lettuce' => [
                    'properties' => [],
                    'condition_lists' => [
                        [
                            'requires' => 'all-of',
                            'conditions' => [
                                [
                                    'type' => 'tag-is-present',
                                    'name' => 'vegetable',
                                ],
                            ],
                        ],
                    ],
                ],
                'tomato' => [
                    'properties' => [],
                    'condition_lists' => [
                        [
                            'requires' => 'all-of',
                            'conditions' => [
                                [
                                    'type' => 'tag-is-present',
                                    'name' => 'vegetable',
                                ],
                            ],
                        ],
                    ],
                ],
            ],
        ]);

        $this->checkDecision(
            $config,
            [],
            ['fruit', 'apple', 'banana'],
            [],
            [
                'valid' => true,
                'invalid_tags' => [],
                'tag_conditions' => [
                    'fruit' => [
                        [
                            'valid' => true,
                            'required' => true,
                            'passed' => 1,
                            'threshold' => 1,
                        ],
                    ],
                    'apple' => [
                        [
                            'valid' => true,
                            'required' => true,
                            'passed' => 1,
                            'threshold' => 1,
                        ],
                    ],
                    'banana' => [
                        [
                            'valid' => true,
                            'required' => true,
                            'passed' => 1,
                            'threshold' => 1,
                        ],
                    ],
                ],
                'tag_group_conditions' => [],
            ],
        );

        $this->checkDecision(
            $config,
            [],
            ['vegetable', 'lettuce'],
            [],
            [
                'valid' => true,
                'invalid_tags' => [],
                'tag_conditions' => [
                    'vegetable' => [
                        [
                            'valid' => true,
                            'required' => true,
                            'passed' => 1,
                            'threshold' => 1,
                        ],
                    ],
                    'lettuce' => [
                        [
                            'valid' => true,
                            'required' => true,
                            'passed' => 1,
                            'threshold' => 1,
                        ],
                    ],
                ],
                'tag_group_conditions' => [],
            ],
        );

        $this->checkDecision(
            $config,
            ['fruit'],
            ['fruit', 'apple', 'lettuce'],
            [],
            [
                'valid' => false,
                'invalid_tags' => [],
                'tag_conditions' => [
                    'fruit' => [
                        [
                            'valid' => true,
                            'required' => true,
                            'passed' => 1,
                            'threshold' => 1,
                        ],
                    ],
                    'apple' => [
                        [
                            'valid' => true,
                            'required' => true,
                            'passed' => 1,
                            'threshold' => 1,
                        ],
                    ],
                    'lettuce' => [
                        [
                            'valid' => false,
                            'required' => true,
                            'passed' => 0,
                            'threshold' => 1,
                        ],
                    ],
                ],
                'tag_group_conditions' => [],
            ],
        );

        $this->checkDecision(
            $config,
            [],
            ['apple'],
            [],
            [
                'valid' => false,
                'invalid_tags' => [],
                'tag_conditions' => [
                    'apple' => [
                        [
                            'valid' => false,
                            'required' => true,
                            'passed' => 0,
                            'threshold' => 1,
                        ],
                    ],
                ],
                'tag_group_conditions' => [],
            ],
        );
    }

    /**
     * Tests the TagEngine's capability with more complicated tag conditions.
     * It also tests the required_if_valid flag.
     *
     * @return void
     */
    public function testComplexDependency(): void
    {
        $config = new TagConfiguration([
            'tags' => [
                // Parent tags
                'fruit' => [
                    'properties' => [],
                    'condition_lists' => [],
                ],
                'two-fruits' => [
                    'properties' => [],
                    'required_if_valid' => true,
                    'condition_lists' => [
                        [
                            'requires' => 'custom',
                            'compare' => 'eq',
                            'value' => 2,
                            'conditions' => [
                                [
                                    'type' => 'tag-is-present',
                                    'name' => 'apple',
                                ],
                                [
                                    'type' => 'tag-is-present',
                                    'name' => 'banana',
                                ],
                                [
                                    'type' => 'tag-is-present',
                                    'name' => 'cherry',
                                ],
                            ],
                        ],
                    ],
                ],
                'fruit-salad' => [
                    'properties' => [],
                    'required_if_valid' => true,
                    'condition_lists' => [
                        [
                            'requires' => 'custom',
                            'compare' => 'gte',
                            'value' => 3,
                            'conditions' => [
                                [
                                    'type' => 'tag-is-present',
                                    'name' => 'apple',
                                ],
                                [
                                    'type' => 'tag-is-present',
                                    'name' => 'banana',
                                ],
                                [
                                    'type' => 'tag-is-present',
                                    'name' => 'cherry',
                                ],
                            ],
                        ],
                    ],
                ],

                // Child tags
                'apple' => [
                    'properties' => [],
                    'condition_lists' => [
                        [
                            'requires' => 'all-of',
                            'conditions' => [
                                [
                                    'type' => 'tag-is-present',
                                    'name' => 'fruit',
                                ],
                            ],
                        ],
                    ],
                ],
                'banana' => [
                    'properties' => [],
                    'condition_lists' => [
                        [
                            'requires' => 'all-of',
                            'conditions' => [
                                [
                                    'type' => 'tag-is-present',
                                    'name' => 'fruit',
                                ],
                            ],
                        ],
                    ],
                ],
                'cherry' => [
                    'properties' => [],
                    'condition_lists' => [
                        [
                            'requires' => 'all-of',
                            'conditions' => [
                                [
                                    'type' => 'tag-is-present',
                                    'name' => 'fruit',
                                ],
                            ],
                        ],
                    ],
                ],
            ],
        ]);

        // Test when two-fruit is required
        $this->checkDecision(
            $config,
            ['fruit', 'apple'],
            ['fruit', 'apple', 'banana'],
            [],
            [
                'valid' => false,
                'invalid_tags' => [],
                'tag_conditions' => [
                    'apple' => [
                        [
                            'valid' => true,
                            'required' => true,
                            'passed' => 1,
                            'threshold' => 1,
                        ],
                    ],
                    'banana' => [
                        [
                            'valid' => true,
                            'required' => true,
                            'passed' => 1,
                            'threshold' => 1,
                        ],
                    ],
                    'two-fruits' => [
                        [
                            'valid' => true,
                            'required' => false,
                            'passed' => 2,
                            'threshold' => 2,
                        ],
                        [
                            'valid' => false,
                            'required' => true,
                            'type' => 'required_if_valid',
                        ],
                    ],
                    'fruit-salad' => [
                        [
                            'valid' => false,
                            'required' => false,
                            'passed' => 2,
                            'threshold' => 3,
                        ],
                    ],
                ],
                'tag_group_conditions' => [],
            ],
        );

        $this->checkDecision(
            $config,
            ['fruit', 'apple'],
            ['fruit', 'apple', 'banana', 'two-fruits'],
            [],
            [
                'valid' => true,
                'invalid_tags' => [],
                'tag_conditions' => [
                    'apple' => [
                        [
                            'valid' => true,
                            'required' => true,
                            'passed' => 1,
                            'threshold' => 1,
                        ],
                    ],
                    'banana' => [
                        [
                            'valid' => true,
                            'required' => true,
                            'passed' => 1,
                            'threshold' => 1,
                        ],
                    ],
                    'two-fruits' => [
                        [
                            'valid' => true,
                            'required' => true,
                            'passed' => 2,
                            'threshold' => 2,
                        ],
                        [
                            'valid' => true,
                            'required' => true,
                            'type' => 'required_if_valid',
                        ],
                    ],
                    'fruit-salad' => [
                        [
                            'valid' => false,
                            'required' => false,
                            'passed' => 2,
                            'threshold' => 3,
                        ],
                    ],
                ],
                'tag_group_conditions' => [],
            ],
        );

        // Test when fruit-salad is required
        $this->checkDecision(
            $config,
            ['fruit', 'apple'],
            ['fruit', 'apple', 'banana', 'cherry'],
            [],
            [
                'valid' => false,
                'invalid_tags' => [],
                'tag_conditions' => [
                    'apple' => [
                        [
                            'valid' => true,
                            'required' => true,
                            'passed' => 1,
                            'threshold' => 1,
                        ],
                    ],
                    'banana' => [
                        [
                            'valid' => true,
                            'required' => true,
                            'passed' => 1,
                            'threshold' => 1,
                        ],
                    ],
                    'cherry' => [
                        [
                            'valid' => true,
                            'required' => true,
                            'passed' => 1,
                            'threshold' => 1,
                        ],
                    ],
                    'two-fruits' => [
                        [
                            'valid' => false,
                            'required' => false,
                            'passed' => 3,
                            'threshold' => 2,
                        ],
                    ],
                    'fruit-salad' => [
                        [
                            'valid' => true,
                            'required' => false,
                            'passed' => 3,
                            'threshold' => 3,
                        ],
                        [
                            'valid' => false,
                            'required' => true,
                            'type' => 'required_if_valid',
                        ],
                    ],
                ],
                'tag_group_conditions' => [],
            ],
        );

        $this->checkDecision(
            $config,
            ['fruit', 'apple'],
            ['fruit', 'apple', 'banana', 'cherry', 'fruit-salad'],
            [],
            [
                'valid' => true,
                'invalid_tags' => [],
                'tag_conditions' => [
                    'apple' => [
                        [
                            'valid' => true,
                            'required' => true,
                            'passed' => 1,
                            'threshold' => 1,
                        ],
                    ],
                    'banana' => [
                        [
                            'valid' => true,
                            'required' => true,
                            'passed' => 1,
                            'threshold' => 1,
                        ],
                    ],
                    'cherry' => [
                        [
                            'valid' => true,
                            'required' => true,
                            'passed' => 1,
                            'threshold' => 1,
                        ],
                    ],
                    'two-fruits' => [
                        [
                            'valid' => false,
                            'required' => false,
                            'passed' => 3,
                            'threshold' => 2,
                        ],
                    ],
                    'fruit-salad' => [
                        [
                            'valid' => true,
                            'required' => true,
                            'passed' => 3,
                            'threshold' => 3,
                        ],
                        [
                            'valid' => true,
                            'required' => true,
                            'type' => 'required_if_valid',
                        ],
                    ],
                ],
                'tag_group_conditions' => [],
            ],
        );

        // Has 'two-fruits', but there are three fruits
        $this->checkDecision(
            $config,
            ['fruit', 'apple'],
            ['fruit', 'apple', 'banana', 'cherry', 'fruit-salad', 'two-fruits'],
            [],
            [
                'valid' => false,
                'invalid_tags' => [],
                'tag_conditions' => [
                    'apple' => [
                        [
                            'valid' => true,
                            'required' => true,
                            'passed' => 1,
                            'threshold' => 1,
                        ],
                    ],
                    'banana' => [
                        [
                            'valid' => true,
                            'required' => true,
                            'passed' => 1,
                            'threshold' => 1,
                        ],
                    ],
                    'cherry' => [
                        [
                            'valid' => true,
                            'required' => true,
                            'passed' => 1,
                            'threshold' => 1,
                        ],
                    ],
                    'two-fruits' => [
                        [
                            'valid' => false,
                            'required' => true,
                            'passed' => 3,
                            'threshold' => 2,
                        ],
                    ],
                    'fruit-salad' => [
                        [
                            'valid' => true,
                            'required' => true,
                            'passed' => 3,
                            'threshold' => 3,
                        ],
                        [
                            'valid' => true,
                            'required' => true,
                            'type' => 'required_if_valid',
                        ],
                    ],
                ],
                'tag_group_conditions' => [],
            ],
        );
    }

    /**
     * Tests the TagEngine with specified tag groups, no conditions.
     *
     * @return void
     */
    public function testTagGroups(): void
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

                'lettuce' => $tag_exists,
                'tomato' => $tag_exists,
            ],
            'tag_groups' => [
                'fruit' => [
                    'members' => ['apple', 'banana', 'cherry'],
                    'condition_lists' => [],
                ],
                'vegetable' => [
                    'members' => ['lettuce', 'tomato'],
                    'condition_lists' => [],
                ],
            ],
        ]);

        $this->checkDecision(
            $config,
            ['apple', 'banana'],
            ['apple', 'lettuce'],
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
            ['tomato'],
            ['banana', 'lettuce'],
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
     * Tests the TagEngine's capability with simple tag group conditions.
     *
     * @return void
     */
    public function testSimpleTagGroupDependency(): void
    {
        // TODO
    }

    /**
     * Tests the TagEngine's capability with more complicated tag group conditions.
     * It also tests the required_if_valid flag.
     *
     * @return void
     */
    public function testComplexTagGroupDependency(): void
    {
        // TODO
    }
}
