source .env
forge script script/Deploy.s.sol:DeployMyContract --rpc-url $SEPOLIA_RPC_URL --broadcast --skip-simulation -vvv
