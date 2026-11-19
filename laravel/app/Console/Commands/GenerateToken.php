<?php

namespace App\Console\Commands;

use App\Models\User;
use Illuminate\Console\Command;
use Illuminate\Support\Str;

class GenerateToken extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'generate:token';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Generate Api Token';

    /**
     * Execute the console command.
     */
    public function handle()
    {
        $user = User::query()
            ->firstOrCreate([
                'name' => 'OpenAlbion',
                'email' => 'openalbion@example.com',
            ]);
        $apiToken = $user->apiTokens()->create([
            'name' => Str::random(6),
            'token' => Str::uuid(),
        ]);

        $this->info('Api Token: '.$apiToken->token);
    }
}
