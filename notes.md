https://webrtc.github.io/samples/

https://imagineer.in/blog/https-on-localhost-with-nginx/
openssl req -x509 -sha256 -nodes -newkey rsa:2048 -days 365 -keyout localhost.key -out localhost.crt
Common Name: localhost
openssl x509 -text -noout -in localhost.crt