FROM nginx

RUN mkdir -p /usr/share/nginx/html/client/dist
COPY src/js/client/dist/ /usr/share/nginx/html/client/

RUN mkdir -p /usr/share/nginx/html/explorer/dist
COPY src/js/explorer/dist/* /usr/share/nginx/html/explorer/dist/

COPY docker/nginx/nginx.conf /etc/nginx/nginx.conf
