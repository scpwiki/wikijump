<?php

namespace Tests\Feature;

use Illuminate\Support\Facades\Hash;
use Wikijump\Models\User;
use Illuminate\Auth\Notifications\ResetPassword;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Illuminate\Support\Facades\Notification;
use Laravel\Fortify\Features;
use Tests\TestCase;

class PasswordResetTest extends TestCase
{
    use RefreshDatabase;

    public function test_reset_password_link_screen_can_be_rendered()
    {
        if (!Features::enabled(Features::updatePasswords())) {
            return $this->markTestSkipped('Password updates are not enabled.');
        }

        $response = $this->get(route('password.request'));

        $response->assertStatus(200);
    }

    public function test_reset_password_link_can_be_requested()
    {
        if (!Features::enabled(Features::updatePasswords())) {
            return $this->markTestSkipped('Password updates are not enabled.');
        }

        Notification::fake();

        $user = User::factory()->create();

        $response = $this->post(route('password.email'), [
            'email' => $user->email,
        ]);

        Notification::assertSentTo($user, ResetPassword::class);
    }

    public function test_reset_password_screen_can_be_rendered()
    {
        if (!Features::enabled(Features::updatePasswords())) {
            return $this->markTestSkipped('Password updates are not enabled.');
        }

        Notification::fake();

        $user = User::factory()->create();

        $response = $this->post(route('password.email'), [
            'email' => $user->email,
        ]);

        Notification::assertSentTo($user, ResetPassword::class, function ($notification) {
            $response = $this->get(route('password.reset', $notification->token));

            $response->assertStatus(200);

            return true;
        });
    }

    public function test_password_can_be_reset_with_valid_token()
    {
        if (!Features::enabled(Features::updatePasswords())) {
            return $this->markTestSkipped('Password updates are not enabled.');
        }

        Notification::fake();

        $user = User::factory()->create();

        $response = $this->post(route('password.email'), [
            'email' => $user->email,
        ]);

        $new_password = Hash::make(bin2hex(random_bytes(32)));

        Notification::assertSentTo($user, ResetPassword::class, function (
            $notification
        ) use ($user, $new_password) {
            $response = $this->post(route('password.update'), [
                'token' => $notification->token,
                'email' => $user->email,
                'password' => $new_password,
                'password_confirmation' => $new_password,
            ]);

            $response->assertSessionHasNoErrors();

            return true;
        });
    }
}
