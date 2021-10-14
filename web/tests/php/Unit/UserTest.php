<?php
declare(strict_types=1);

namespace Tests\Unit;

use Exception;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Tests\TestCase;
use Wikijump\Models\User;

class UserTest extends TestCase
{
    use RefreshDatabase;

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

    /**
     * Verifying that the settings getter method works.
     * @return void
     */
    public function testGet()
    {
        /**
         * This test and others assumes that the default setting for
         * `allow_pms` is true.
         */
        $user = User::factory()->create();
        $this->assertTrue($user->get('allow_pms'));
        /**
         * Note this will work even when there is no extant Settings object
         * for a user.
         */
        $this->assertEquals(0, $user->settings()->count());
        $this->assertFalse($user->settings()->exists());

        $user->set(['allow_pms' => false]);
        $this->assertFalse($user->get('allow_pms'));
        $this->assertEquals(1, $user->settings()->count());
        $this->assertTrue($user->settings()->exists());
    }

    /**
     * Verifying the settings setter works as expected.
     * @return void
     */
    public function testSet()
    {
        $user = User::factory()->create();
        $this->assertFalse($user->settings()->exists());

        $this->assertTrue($user->get('allow_pms'));

        $user->set(['allow_pms' => false]);
        $this->assertTrue($user->settings()->exists());

        $this->assertFalse($user->get('allow_pms'));

        /**
         * A note on the difference between these two similar-looking calls.
         * The first one checks for the number of Settings objects attached to a
         *  user, which should always be 0 or 1.
         * The second one checks for the number of individual settings within
         *  the Setting object, which can be any positive number, but not 0,
         *  as you'd then be trying to count(null) which makes PHP sad.
         */
        $this->assertEquals(1, $user->settings()->count());
        $this->assertEquals(1, count($user->settings->settings));

        $user->set(['show_last_online_time' => false]);
        $this->assertEquals(1, $user->settings()->count());
        $user->refresh();
        $this->assertEquals(2, count($user->settings->settings));

        $user->set(['show_last_online_time' => true]);
        $user->refresh();
        $this->assertEquals(1, $user->settings()->count());
        $this->assertEquals(1, count($user->settings->settings));

        $user->set(['allow_pms' => true]);
        $this->assertEquals(0, $user->settings()->count());
        $this->assertFalse($user->settings()->exists());
    }

    public function test_exception_for_setting_without_default()
    {
        $user = User::factory()->create();
        $this->expectException(Exception::class);
        $this->expectExceptionMessage(
            'phpunit_invalid_default does not have a default set',
        );
        $user->set(['phpunit_invalid_default' => true]);
    }
}
