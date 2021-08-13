<?php

namespace Tests\Feature;

use Illuminate\Support\Facades\Hash;
use Wikijump\Models\User;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Laravel\Jetstream\Features;
use Laravel\Jetstream\Http\Livewire\DeleteUserForm;
use Livewire\Livewire;
use Tests\TestCase;

class DeleteAccountTest extends TestCase
{
    use RefreshDatabase;

    public function test_user_accounts_can_be_deleted()
    {
        if (! Features::hasAccountDeletionFeatures()) {
            return $this->markTestSkipped('Account deletion is not enabled.');
        }

        $this->actingAs($user = User::factory()->create(['password' => Hash::make('password')]));

        $component = Livewire::test(DeleteUserForm::class)
                        ->set('password', 'password')
                        ->call('deleteUser');

        $this->assertTrue($user->fresh()->trashed());
    }

    public function test_correct_password_must_be_provided_before_account_can_be_deleted()
    {
        if (! Features::hasAccountDeletionFeatures()) {
            return $this->markTestSkipped('Account deletion is not enabled.');
        }

        $this->actingAs($user = User::factory()->create());

        Livewire::test(DeleteUserForm::class)
                        ->set('password', Hash::make(bin2hex(random_bytes(32))))
                        ->call('deleteUser')
                        ->assertHasErrors(['password']);

        $this->assertNotNull($user->fresh());
    }
}
