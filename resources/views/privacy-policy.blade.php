<!DOCTYPE html>
<html lang="{{ str_replace('_', '-', app()->getLocale()) }}">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">

    <title>Privacy Policy | OpenAlbion</title>

    <!-- Fonts -->
    <link rel="preconnect" href="https://fonts.bunny.net">
    <link href="https://fonts.bunny.net/css?family=figtree:400,600&display=swap" rel="stylesheet" />

    <!-- Scripts -->
    @vite(['resources/css/app.css', 'resources/js/app.js'])
</head>
<body>
    <div class="min-h-screen bg-gray-50 py-8 flex flex-col justify-center relative overflow-hidden lg:py-12">
        <div class="relative w-full px-6 py-12 bg-white md:max-w-3xl md:mx-auto lg:max-w-4xl lg:pt-16 lg:pb-28">
            <div class="mt-8 prose prose-slate mx-auto lg:prose-lg">
                <h2>Privacy Policy</h2>

                <p>OpenAlbion provides an Albion Online data API for developers to build their own applications. When using our service, we may collect your name, email address, and avatar from your Google login.</p>

                <p>We use this information to provide and improve our service, notify you about changes, and for other related purposes.</p>

                <p>We do not share your personal information with third parties, except as required by law or to protect our rights.</p>

                <p>Our service may contain links to other sites that are not operated by us. We have no control over and assume no responsibility for the content, privacy policies, or practices of any third-party sites or services.</p>

                <p>By using our service, you consent to our privacy policy.</p>
            </div>
        </div>
    </div>
</body>
</html>
