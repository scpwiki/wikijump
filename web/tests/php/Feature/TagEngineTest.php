<?php
declare(strict_types=1);

namespace Tests\Feature;

use Tests\TestCase;
use Ds\Set;
use Wikijump\Services\TagEngine\TagConfiguration;
use Wikijump\Services\TagEngine\TagEngine;

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
    public function testAllowedTagConfiguration(): void
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
}
