<x-app-layout>
    <x-slot name="header">
        <h2 class="font-semibold text-xl text-gray-800 leading-tight">
            {{ __('Api Tokens') }}
        </h2>
    </x-slot>

    <div class="py-12">
        <div class="max-w-7xl mx-auto sm:px-6 lg:px-8 space-y-6">
            <div class="p-4 sm:p-8 bg-white shadow sm:rounded-lg">
                <div class="max-w-xl">
                    <section>
                        <header>
                            <h2 class="text-lg font-medium text-gray-900">
                                {{ __('Create Api Token') }}
                            </h2>

                            <p class="mt-1 text-sm text-gray-600">
                                You can create a maximum of {{ config('settings.maximum_api_token') }} API tokens. Each API token has a rate limit of 60 calls per minute.
                            </p>
                            <p class="mt-1 text-sm text-gray-600">
                                If you find that OpenAlbion has been useful for your project, please consider mentioning OpenAlbion on your project.
                            </p>
                        </header>

                        <form method="post" action="{{ route('apiTokens.store') }}" class="mt-6 space-y-6">
                            @csrf

                            <div>
                                <x-input-label for="name" :value="__('Name')" />
                                <x-text-input id="name" name="name" type="text" class="mt-1 block w-full" required autofocus autocomplete="name" :disabled="$tokens->count() >= config('settings.maximum_api_token')" />
                                <x-input-error class="mt-2" :messages="$errors->get('name')" />
                            </div>

                            <div class="flex items-center gap-4">
                                @if($tokens->count() >= config('settings.maximum_api_token')) 
                                    <x-primary-button class="disabled:opacity-25" disabled>{{ __('Save') }}</x-primary-button>
                                @else
                                    <x-primary-button>{{ __('Create') }}</x-primary-button>
                                @endif
                                
                            </div>
                        </form>
                    </section>
                </div>
            </div>
            @if($tokens->count())
            <div class="mt-4 bg-white overflow-hidden shadow sm:rounded-lg">
                <div class="flex flex-col">
                    <div class="-my-2 overflow-x-auto sm:-mx-6 lg:-mx-8">
                        <div
                            class="py-2 align-middle inline-block min-w-full sm:px-6 lg:px-8"
                        >
                            <div
                                class="shadow overflow-hidden border-b border-gray-200 sm:rounded-lg"
                            >
                                <table
                                    class="min-w-full divide-y divide-gray-200"
                                >
                                    <thead class="bg-gray-50">
                                        <tr>
                                            <th
                                                scope="col"
                                                class="md:w-1/4 px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                                            >
                                                Name
                                            </th>
                                            <th
                                                scope="col"
                                                class="md:w-1/4 px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                                            >
                                                Token
                                            </th>
                                            <th
                                                scope="col"
                                                class="relative px-6 py-3"
                                            >
                                                <span class="sr-only"
                                                    >Action</span
                                                >
                                            </th>
                                        </tr>
                                    </thead>
                                    <tbody
                                        class="bg-white divide-y divide-gray-200"
                                    >
                                        @foreach($tokens as $token)
                                        <tr
                                        >
                                            <td
                                                class="px-6 py-4 whitespace-nowrap"
                                            >
                                                <div
                                                    class="flex items-center"
                                                >
                                                    <div class="ml-4">
                                                        <p
                                                            class="text-sm font-medium text-gray-900"
                                                        >
                                                            {{ $token->name }}
                                                        </p>
                                                    </div>
                                                </div>
                                            </td>
                                            <td
                                                class="px-6 py-4 whitespace-nowrap"
                                            >
                                                <p
                                                    class="text-sm text-gray-900"
                                                >
                                                    {{ $token->token }}
                                                </p>
                                            </td>
                                            <td
                                                class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium"
                                            >
                                                <form method="post" action="{{ route('apiTokens.destroy', ['tokenId' => $token->id]) }}">
                                                    @csrf
                                                    @method('delete')
                                                    <x-danger-button>{{ __('Delete') }}</x-danger-button>
                                                </form>
                                            </td>
                                        </tr>
                                        @endforeach
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
            @endif
        </div>
    </div>
</x-app-layout>
