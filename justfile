dev:
  cargo watch --clear --watch src --exec run

reverse-proxy-mail:
  mitmproxy --mode reverse:http://localhost:8000 --listen-port 8080

reverse-proxy-sms:
  mitmproxy --mode reverse:https://api.ng.termii.com --listen-port 8080
