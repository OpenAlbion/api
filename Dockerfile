FROM serversideup/php:8.3-unit

USER root

RUN apt update -y && apt install npm -y

COPY ./docker/liblibsql_php.so /usr/local/liblibsql_php.so

COPY ./docker/zzz-custom-php.ini /usr/local/etc/php/conf.d/

USER www-data

COPY --chown=www-data:www-data . /var/www/html

RUN composer install --optimize-autoloader --no-dev

RUN npm install \
    && npm run build