docker run --rm -i \
    -p "8000:8000" \
    --name stellar \
    stellar/quickstart:latest \
    --local \
    --limits testnet \
    --enable-soroban-rpc \
    --enable-soroban-diagnostic-events