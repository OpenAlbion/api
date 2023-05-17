<!DOCTYPE html>
<html lang="{{ str_replace('_', '-', app()->getLocale()) }}">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">

    <title>Terms of Service | OpenAlbion</title>

    <meta name="description" content="OpenAlbion is a free and open-source platform providing Albion Online data and an API.">
    <meta name="keywords" content="OpenAlbion, Albion Online, data, API, open-source, developers">
    <meta name="author" content="Pyae Sone Aung">

    <meta property="og:title" content="OpenAlbion">
    <meta property="og:description" content="OpenAlbion is a free and open-source platform providing Albion Online data and an API.">
    <meta property="og:url" content="https://api.openalbion.com">
    <meta property="og:type" content="website">
    <meta property="og:site_name" content="OpenAlbion">

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
                <h2>Terms of Service:</h2>
                <p>By using OpenAlbion's services, you agree to the following terms and conditions:</p>
                <ol>
                    <li>
                      <strong>Usage:</strong> You are responsible for complying with all applicable laws and regulations while using OpenAlbion's services. Any activities conducted using the provided data and API are solely your responsibility.
                    </li>
                    <li>
                      <strong>Data Accuracy:</strong> We strive to provide accurate data sourced from the Albion Online Wiki. However, we cannot guarantee its completeness or correctness. Any inaccuracies or errors in the database are our responsibility, not related to the Albion Online Wiki.
                    </li>
                    <li>
                      <strong>API Usage:</strong> Our API offers free and unlimited access to Albion Online data. We may enforce reasonable rate limits or other restrictions to ensure fair usage and protect service stability.
                    </li>
                    <li>
                      <strong>Self-Hosting:</strong> You have the option to self-host the OpenAlbion project by cloning the repository. You are responsible for setting it up and maintaining it on your own server.
                    </li>
                    <li>
                      <strong>Disclaimer:</strong> OpenAlbion is provided "as is" without any warranties or guarantees. We do not assume any liability for damages, losses, or disruptions caused by the use of our services or reliance on the provided data.
                    </li>
                    <li>
                      <strong>Changes:</strong> These terms may be updated without prior notice. It is your responsibility to review the most current version.
                    </li>
                </ol>
                <p>
                    By using OpenAlbion's services, you acknowledge and agree to these terms.
                </p>
                <p>
                    Please note that this simplified text may need further customization and should not be considered as legal advice. Consult with a legal professional to ensure compliance with applicable laws and regulations.
                </p>
            </div>
        </div>
    </div>
</body>
</html>
