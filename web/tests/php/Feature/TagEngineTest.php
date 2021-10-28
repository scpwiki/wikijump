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
                'failed_tag_conditions' => [],
                'failed_tag_group_conditions' => [],
            ],
        );
    }

    public function testAllowedTagConfiguration(): void
    {
        // TODO
    }
}
