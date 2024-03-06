FROM php:8.3-alpine

COPY --from=mlocati/php-extension-installer /usr/bin/install-php-extensions /usr/local/bin/
COPY --from=composer:latest /usr/bin/composer /usr/local/bin/composer

RUN install-php-extensions pcntl sockets exif sqlite3

RUN apk add npm

COPY . /var/www

WORKDIR /var/www

RUN wget -O /usr/local/bin/frankenphp https://github.com/dunglas/frankenphp/releases/download/v1.1.0/frankenphp-linux-x86_64 && chmod +x /usr/local/bin/frankenphp

RUN composer install --no-dev && \
    npm install && \
    npm run build

RUN php artisan key:generate && \
    php artisan optimize

ENTRYPOINT ["php", "artisan", "octane:start", "--server=frankenphp", "--port=80", "--host=0.0.0.0", "--admin-port=2019"]
