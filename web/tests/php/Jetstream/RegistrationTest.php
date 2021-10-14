<?php

namespace Tests\Feature;

use Illuminate\Support\Facades\Hash;
use Wikidot\Utils\WDStringUtils;
use Wikijump\Providers\RouteServiceProvider;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Laravel\Fortify\Features;
use Laravel\Jetstream\Jetstream;
use Tests\TestCase;

class RegistrationTest extends TestCase
{
    use RefreshDatabase;

    public function test_registration_screen_can_be_rendered()
    {
        if (!Features::enabled(Features::registration())) {
            return $this->markTestSkipped('Registration support is not enabled.');
        }

        $response = $this->get('/register');

        $response->assertStatus(200);
    }

    public function test_registration_screen_cannot_be_rendered_if_support_is_disabled()
    {
        if (Features::enabled(Features::registration())) {
            return $this->markTestSkipped('Registration support is enabled.');
        }

        $response = $this->get('/register');

        $response->assertStatus(404);
    }

    public function test_new_users_can_register()
    {
        if (!Features::enabled(Features::registration())) {
            return $this->markTestSkipped('Registration support is not enabled.');
        }

        $new_password = bin2hex(random_bytes(32));

        $response = $this->post('/user--services/register', [
            'username' => 'Test User',
            'email' => 'test@example.com',
            'password' => $new_password,
            'password_confirmation' => $new_password,
            'terms' => Jetstream::hasTermsAndPrivacyPolicyFeature(),
        ]);
        $this->assertAuthenticated();
        $response->assertRedirect(RouteServiceProvider::HOME);
    }
}
