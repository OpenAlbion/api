<?php

namespace App\Http\Controllers;

use App\Http\Requests\ApiTokenCreateRequest;
use App\Models\ApiToken;
use Illuminate\Support\Str;

class DashboardController extends Controller
{
    public function index()
    {
        $tokens = ApiToken::query()
            ->where('user_id', auth()->id())
            ->get();

        return view('dashboard', ['tokens' => $tokens]);
    }

    public function storeApiToken(ApiTokenCreateRequest $request)
    {
        $existingTokens = ApiToken::query()
            ->where('user_id', auth()->id())
            ->count();
        if ($existingTokens < 10) {
            ApiToken::query()
                ->create(array_merge($request->validated(), [
                    'token' => Str::uuid(),
                    'user_id' => auth()->id(),
                ]));
        }

        return redirect()->route('dashboard');
    }

    public function destroyApiToken($tokenId)
    {
        $apiToken = ApiToken::query()
            ->where('user_id', auth()->id())
            ->findOrFail($tokenId);
        cache()->forget($apiToken->token);
        $apiToken->delete();

        return redirect()->route('dashboard');
    }
}
