require("dotenv").config();

const { nativeToScVal, Address, Keypair } = require("@stellar/stellar-sdk");

const { users, minDelay, sleep, generateNewKeypair } = require("../common");
const {
  scheduleOperation,
  executeOperation,
  invokeContract,
} = require("./common");

async function scheduleAndExecuteMint() {
  const user = generateNewKeypair();
  let timeLockContractId = process.env.timelock_contract_id;
  let proposerLabel = users.proposer;
  let proposerSecret = process.env[`${proposerLabel}_secret`];
  let proposer = process.env[`${proposerLabel}_pubkey`];
  let target = process.env.mock_token_contract_id;
  let fnName = "mint";
  let amount = 1000000000;
  let data = [
    nativeToScVal(user, { type: "address" }),
    nativeToScVal(amount, { type: "i128" }),
  ];
  let currentTimeStamp = new Date().getTime() + "";
  let saltString = Buffer.from(currentTimeStamp)
    .toString("hex")
    .padStart(64, "0");
  console.log("saltString", saltString);
  let salt = Buffer.from(saltString, "hex");
  let delay = minDelay + 2;

  const scheduleParams = [
    nativeToScVal(Address.fromString(proposer), { type: "address" }),
    nativeToScVal(Address.fromString(target), { type: "address" }),
    nativeToScVal(fnName, { type: "symbol" }),
    nativeToScVal(nativeToScVal(data)),
    nativeToScVal(salt, { type: "bytes" }),
    nativeToScVal(null),
    nativeToScVal(delay, { type: "u64" }),
  ];

  const keyPair = Keypair.fromSecret(proposerSecret);
  const result = await scheduleOperation(
    keyPair,
    timeLockContractId,
    scheduleParams
  );
  if (result) {
    await sleep(20000);
    let executor = process.env[`${users.executor}_pubkey`];
    let executorSecret = process.env[`${users.executor}_secret`];
    let executorKeyPair = Keypair.fromSecret(executorSecret);
    const executeParams = scheduleParams.slice(1, scheduleParams.length - 1);
    executeParams.unshift(
      nativeToScVal(Address.fromString(executor), { type: "address" })
    );

    let balance = await invokeContract(executorKeyPair, target, "balance", [
      nativeToScVal(user, { type: "address" }),
    ]);
    console.log("before mint balance: ", balance);
    await executeOperation(executorKeyPair, timeLockContractId, executeParams);
    balance = await invokeContract(executorKeyPair, target, "balance", [
      nativeToScVal(user, { type: "address" }),
    ]);
    console.log("after mint balance: ", balance);
  }
}

async function scheduleAndExecuteMintWithPredecessor() {
  const user = generateNewKeypair();
  const user2 = generateNewKeypair();
  let timeLockContractId = process.env.timelock_contract_id;
  let proposerLabel = users.proposer;
  let proposerSecret = process.env[`${proposerLabel}_secret`];
  let proposer = process.env[`${proposerLabel}_pubkey`];
  let target = process.env.mock_token_contract_id;
  let fnName = "mint";
  let amount = 1000000000;
  let data = [
    nativeToScVal(user, { type: "address" }),
    nativeToScVal(amount, { type: "i128" }),
  ];
  let data2 = [
    nativeToScVal(user2, { type: "address" }),
    nativeToScVal(amount, { type: "i128" }),
  ];
  let currentTimeStamp = new Date().getTime() + "";
  let saltString = Buffer.from(currentTimeStamp)
    .toString("hex")
    .padStart(64, "0");
  console.log("saltString", saltString);
  let salt = Buffer.from(saltString, "hex");
  let delay = minDelay + 2;

  const scheduleParams1 = [
    nativeToScVal(Address.fromString(proposer), { type: "address" }),
    nativeToScVal(Address.fromString(target), { type: "address" }),
    nativeToScVal(fnName, { type: "symbol" }),
    nativeToScVal(nativeToScVal(data)),
    nativeToScVal(salt, { type: "bytes" }),
    nativeToScVal(null),
    nativeToScVal(delay, { type: "u64" }),
  ];

  const keyPair = Keypair.fromSecret(proposerSecret);
  const { result, operationId } = await scheduleOperation(
    keyPair,
    timeLockContractId,
    scheduleParams1
  );

  if (result) {
    const scheduleParams2 = [
      nativeToScVal(Address.fromString(proposer), { type: "address" }),
      nativeToScVal(Address.fromString(target), { type: "address" }),
      nativeToScVal(fnName, { type: "symbol" }),
      nativeToScVal(nativeToScVal(data2)),
      nativeToScVal(salt, { type: "bytes" }),
      nativeToScVal(Buffer.from(operationId, "hex"), { type: "bytes" }),
      nativeToScVal(delay, { type: "u64" }),
    ];
    const { result: result2 } = await scheduleOperation(
      keyPair,
      timeLockContractId,
      scheduleParams2
    );

    await sleep(20000);
    let executor = process.env[`${users.executor}_pubkey`];
    let executorSecret = process.env[`${users.executor}_secret`];
    let executorKeyPair = Keypair.fromSecret(executorSecret);
    const executeParams = scheduleParams1.slice(1, scheduleParams1.length - 1);
    executeParams.unshift(
      nativeToScVal(Address.fromString(executor), { type: "address" })
    );
    await executeOperation(executorKeyPair, timeLockContractId, executeParams);
    if (result2) {
      const executeParams2 = scheduleParams2.slice(
        1,
        scheduleParams2.length - 1
      );
      executeParams2.unshift(
        nativeToScVal(Address.fromString(executor), { type: "address" })
      );
      await executeOperation(
        executorKeyPair,
        timeLockContractId,
        executeParams2
      );
    }
  }
}

async function cancelMintOperation() {
  const user = generateNewKeypair();
  let timeLockContractId = process.env.timelock_contract_id;
  let cancellerLabel = users.proposer;
  let cancellerSecret = process.env[`${cancellerLabel}_secret`];
  let proposer = process.env[`${cancellerLabel}_pubkey`];
  let target = process.env.mock_token_contract_id;
  let fnName = "mint";
  let amount = 1000000000;
  let data = [
    nativeToScVal(user, { type: "address" }),
    nativeToScVal(amount, { type: "i128" }),
  ];
  let currentTimeStamp = new Date().getTime() + "";
  let saltString = Buffer.from(currentTimeStamp)
    .toString("hex")
    .padStart(64, "0");
  console.log("saltString", saltString);
  let salt = Buffer.from(saltString, "hex");
  let delay = minDelay + 2;

  const scheduleParams = [
    nativeToScVal(Address.fromString(proposer), { type: "address" }),
    nativeToScVal(Address.fromString(target), { type: "address" }),
    nativeToScVal(fnName, { type: "symbol" }),
    nativeToScVal(nativeToScVal(data)),
    nativeToScVal(salt, { type: "bytes" }),
    nativeToScVal(null),
    nativeToScVal(delay, { type: "u64" }),
  ];

  const keyPair = Keypair.fromSecret(cancellerSecret);
  const { result, operationId } = await scheduleOperation(
    keyPair,
    timeLockContractId,
    scheduleParams
  );
  if (result) {
    let cancelId = Buffer.from(operationId, "hex");
    let srValCancelId = nativeToScVal(cancelId, { type: "bytes" });
    let timelock = await invokeContract(
      keyPair,
      timeLockContractId,
      "get_schedule_lock_time",
      [srValCancelId]
    );
    console.log(`before cancel operation timelock : ${timelock}`);
    await invokeContract(keyPair, timeLockContractId, "cancel", [
      nativeToScVal(Address.fromString(proposer), { type: "address" }),
      srValCancelId,
    ]);
    timelock = await invokeContract(
      keyPair,
      timeLockContractId,
      "get_schedule_lock_time",
      [srValCancelId]
    );
    console.log(`after cancel operation timelock : ${timelock}`);
  }
}

async function main() {
  await scheduleAndExecuteMint();
  await scheduleAndExecuteMintWithPredecessor();
  await cancelMintOperation();
}

module.exports = {
  tokenMint: main,
};
