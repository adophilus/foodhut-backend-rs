dev:
  cargo watch --clear --watch src --exec run

reverse-proxy:
  mitmproxy --mode reverse:http://localhost:8000 --listen-port 8080

