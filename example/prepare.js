const {Horizon, Keypair, Networks} = require("@stellar/stellar-sdk");
const fs = require("fs");

const {cmd, getRootPath, users, minDelay, networks} = require("./common");

const horizonUrl = "http://localhost:8000";
const horizon = new Horizon.Server(horizonUrl, {allowHttp: true});

(async () => {
    await cmd(
      `soroban network add ${networks.name} --rpc-url ${horizonUrl}/soroban/rpc --network-passphrase '${Networks.STANDALONE}'`
    );
    console.log(`added ${networks.name}`);

    let deployer = users.deployer;
    let timelockOwner = users.timelockOwner;
    let proposer = users.proposer;
    let executor = users.executor;;
    let cancaller = users.cancaller;

    if (fs.existsSync(`${getRootPath()}/example/.env`)) {
        fs.unlinkSync(`${getRootPath()}/example/.env`);
    }
    let file = ``;

    let deployerPubKey = await createAccount(deployer);
    let timelockOwnerPubKey = await createAccount(timelockOwner);
    let proposerPubKey = await createAccount(proposer);
    let executorPubKey = await createAccount(executor);
    let cancallerPubKey = await createAccount(cancaller);

    let tokenContractId = await deployContract(`${getRootPath()}/example/wasm/soroban_token_contract.optimized.wasm`, "mock_token");
    let timelockContractId = await deployContract(`${getRootPath()}/target/wasm32-unknown-unknown/release/time_lock.optimized.wasm`, "timelock");
    let timelockContractId2 = await deployContract(`${getRootPath()}/target/wasm32-unknown-unknown/release/time_lock.optimized.wasm`, "timelock_self_managed");

    await cmd(
        `soroban contract invoke --id ${tokenContractId} --network ${networks.name} --source ${deployer} -- initialize --admin ${timelockContractId} --decimal 6 --name USDC --symbol USDC`
      );
    console.log("initialized mock_token");

    await cmd(
        `soroban contract invoke --id ${timelockContractId} --network ${networks.name} --source ${deployer} -- initialize --min_delay ${minDelay} --proposers '{"vec":[{"address":"${proposerPubKey}"}]}' --executors '{"vec":[{"address":"${executorPubKey}"}]}' --owner ${null}`
    );
    console.log("initialized time_lock");

    await cmd(
        `soroban contract invoke --id ${timelockContractId2} --network ${networks.name} --source ${deployer} -- initialize --min_delay ${minDelay} --proposers '{"vec":[{"address":"${proposerPubKey}"}]}' --executors '{"vec":[{"address":"${executorPubKey}"}]}' --owner ${timelockOwnerPubKey}`
    );
    console.log("initialized time_lock_self_managed");

    console.log("file", file);
    fs.writeFileSync(`${getRootPath()}/example/.env`, file);
    console.log("✅");

    async function createAccount(name) {
        const keypair = Keypair.random();
        try {
          await horizon.friendbot(keypair.publicKey()).call();
        } catch {
          throw new Error(
            `Issue with ${keypair.publicKey()} account. Ensure you're running the \`./docker.sh\` network.`
          );
        }
        process.env.SOROBAN_SECRET_KEY = keypair.secret();
        await cmd(`soroban keys add ${name} --secret-key`);
        console.log(`created account ${name}: ${keypair.publicKey()}`);
        file += `${name}_secret=${keypair.secret()}\n`;
        file += `${name}_pubkey=${keypair.publicKey()}\n`;
        return keypair.publicKey();
    }

    async function deployContract(wasm, name) {
        console.log(`wasm: ${wasm}`);
        const contractId = await cmd(
          `soroban contract deploy --wasm ${wasm} --network ${networks.name} --source ${deployer}`
        );
        if (!contractId) throw new Error(`Contract ${name} not deployed`);
        console.log(`deployed contract ${name}: ${contractId}`);
        file += `${name}_contract_id=${contractId}\n`;
        return (contractId).replace(/\W/g, "");
    }
    
    process.exit(0);
})();