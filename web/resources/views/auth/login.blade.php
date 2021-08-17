<x-guest-layout>
    <x-jet-authentication-card>
        <x-slot name="logo">
            <x-jet-authentication-card-logo />
        </x-slot>

        <x-jet-validation-errors class="mb-4" />

        @if (session('status'))
            <div class="mb-4 font-medium text-sm text-green-600">
                {{ session('status') }}
            </div>
        @endif
        @php
            if (empty(session('backUrl')))
            {
               session(['backUrl' => $_SERVER['HTTP_REFERER']]);
            }
           session()->reflash();
        @endphp
        <form method="POST" action="{{ route('login') }}">
            @csrf

            <div>
                <x-jet-label for="username" value="{{ __('Username') }}" />
                <x-jet-input id="username" class="block mt-1 w-full" type="text" name="username" :value="old('username')" required autofocus />
            </div>

            <div class="mt-4">
                <x-jet-label for="password" value="{{ __('Password') }}" />
                <x-jet-input id="password" class="block mt-1 w-full" type="password" name="password" required autocomplete="current-password" />
            </div>

            <div class="block mt-4">
                <label for="remember_me" class="flex items-center">
                    <x-jet-checkbox id="remember_me" name="remember" />
                    <span class="ml-2 text-sm text-gray-600">{{ __('Remember me') }}</span>
                </label>
            </div>
            <div class="block mt-4">
                <x-jet-button class="ml-4">
                    {{ __('Log in') }}
                </x-jet-button>
            </div>

            @if (Route::has('password.request'))
            <div class="flex items-center justify-end">
                <a class="underline text-sm text-gray-600 hover:text-gray-900" href="{{ route('password.request') }}">
                    {{ __('Forgot your password?') }}
                </a>
            </div>
            @endif
            <div class="flex items-center justify-end mt-2">
                <a class="underline text-sm text-gray-600 hover:text-gray-900" href="{{ route('register') }}">
                    {{ __('Create a new account') }}
                </a>
            </div>
        </form>
    </x-jet-authentication-card>
</x-guest-layout>
