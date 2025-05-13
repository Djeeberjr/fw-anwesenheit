#!/usr/bin/env bash

generate_random_hex() {
  openssl rand -hex 4 | tr '[:lower:]' '[:upper:]'
}

output_hotspot_id() {
  echo "[+] UID.... C1532B57"
}

trap 'output_hotspot_id' SIGUSR1

echo "Proxmark 3 mock script"
echo "Outputs a random ID every 1 to 5 seconds"

while true; do
  random_id=$(generate_random_hex)
  echo "[+] UID.... $random_id"
  if (( RANDOM % 2 == 0 )); then
    echo "[+] UID.... $random_id"
  fi
  sleep "$((RANDOM % 5 + 1))"
done
