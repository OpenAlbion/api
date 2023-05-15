<?php

namespace App\Http\Controllers\Auth;

use App\Http\Controllers\Controller;
use App\Models\User;
use Illuminate\Support\Facades\Auth;
use Laravel\Socialite\Facades\Socialite;

class GoogleSignInController extends Controller
{
    public function redirect()
    {
        return Socialite::driver('google')->redirect();
    }

    public function callback()
    {
        $googleUser = Socialite::driver('google')->user();

        $user = User::updateOrCreate(
            [
                'social_id' => $googleUser->id,
                'social_provider' => 'google',
            ],
            [
                'name' => $googleUser->name,
                'email' => $googleUser->email,
                'social_token' => $googleUser->token,
                'social_refresh_token' => $googleUser->refreshToken,
            ]
        );

        Auth::login($user);

        return redirect('/dashboard');
    }
}
