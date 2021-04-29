<?php

namespace Tests\Unit;

use Tests\TestCase;
use Wikijump\Models\User;

class UserTest extends TestCase
{
    /**
     * A basic test of the factory.
     *
     * @return void
     */
    public function test_models_can_be_instantiated()
    {
        $user = User::factory()->make();
        $this->assertTrue($user instanceof User);
    }
}
