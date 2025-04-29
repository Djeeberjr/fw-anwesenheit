# fw-anwesenheit

# Config 

Environment variables:

- `PM3_BIN`: Path to the pm3 binary. Seach in path if not set. Can also be set to the `pm3_mock.sh` for testing.
- `LOG_LEVEL`: Can be set to either "debug","warn","error","trace" or "info". Defaults to "warn" in production.
- `HTTP_PORT`: What port to listen on. Defaults to 80.
