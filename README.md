# OpenAlbion

OpenAlbion is a free and open-source platform providing Albion Online data and an API.

![OpenAlbion Preview](https://res.cloudinary.com/pyaesoneaung/image/upload/v1685124262/openalbion/preview.png)

## Documentation

Read the full documentation at [openalbion.com](https://openalbion.com).

## Self-host

Self-hosting our project is easy - simply clone the repository and set it up on your own server.

### Installation

Our project is built using Laravel framework, making it easy for you to set up just like any other Laravel project.

```bash
git clone https://github.com/OpenAlbion/api.git openalbion
cd openalbion
composer install
cp .env.example .env
php artisan key:generate
php artisan migrate
```

## License

The OpenAlbion is open-sourced software licensed under the [MIT license](https://opensource.org/license/mit/).
