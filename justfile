dev:
  cargo watch --clear --watch src --exec run

# reverse-proxy:
#   mitmproxy --mode reverse:http://localhost:8000 --listen-port 8080

reverse-proxy:
  mitmproxy --mode reverse:https://api.ng.termii.com --listen-port 8080
