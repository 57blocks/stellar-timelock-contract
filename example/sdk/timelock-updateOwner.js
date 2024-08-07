require("dotenv").config();

const {
  nativeToScVal,
  Address,
  Keypair,
} = require("@stellar/stellar-sdk");

const { users, minDelay, sleep } = require("../common");
const {
  scheduleOperation,
  executeOperation,
  invokeContract,
} = require("./common");

async function scheduleAndExecuteUpdateOwner(newOwner, newOwnerSecret) {
  let timeLockContractId = process.env.timelock_contract_id;
  let proposerLabel = users.proposer;
  let proposerSecret = process.env[`${proposerLabel}_secret`];
  let proposer = process.env[`${proposerLabel}_pubkey`];
  let target = process.env.timelock_contract_id;
  let fnName = "update_owner";
  let data = [nativeToScVal(newOwner, { type: "address" })];
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
  const {result} = await scheduleOperation(
    keyPair,
    timeLockContractId,
    scheduleParams
  );

  let executor = process.env[`${users.executor}_pubkey`];
  let executorSecret = process.env[`${users.executor}_secret`];
  let executorKeyPair = Keypair.fromSecret(executorSecret);
  if (result) {
    await sleep(20000);
    const executeParams = scheduleParams.slice(1, scheduleParams.length - 1);
    executeParams.unshift(
      nativeToScVal(Address.fromString(executor), { type: "address" })
    );
    await executeOperation(executorKeyPair, timeLockContractId, executeParams);
  }

  // rollback to the original owner
    let timeLockOwnerLabel = users.timelockOwner;
    let oldOwner = process.env[`${timeLockOwnerLabel}_pubkey`];
    data = [nativeToScVal(oldOwner, { type: "address" })];
    const rollbackScheduleParams = [
      nativeToScVal(Address.fromString(proposer), { type: "address" }),
      nativeToScVal(Address.fromString(target), { type: "address" }),
      nativeToScVal(fnName, { type: "symbol" }),
      nativeToScVal(nativeToScVal(data)),
      nativeToScVal(salt, { type: "bytes" }),
      nativeToScVal(null),
      nativeToScVal(delay, { type: "u64" }),
    ];
    
    const {result: rollbackResult} = await scheduleOperation(
      keyPair,
      timeLockContractId,
      rollbackScheduleParams
    );

    if (rollbackResult) {
      await sleep(20000);
      const rollbackExecuteParams = rollbackScheduleParams.slice(1, rollbackScheduleParams.length - 1);
      rollbackExecuteParams.unshift(
      nativeToScVal(Address.fromString(executor), { type: "address" })
    );
      await executeOperation(executorKeyPair, timeLockContractId, rollbackExecuteParams);
    }
}

async function updateOwnerDirectly(newOwner, newOwnerSecret) {
  let timeLockOwnerLabel = users.timelockOwner;
  let timeLockOwnerSecret = process.env[`${timeLockOwnerLabel}_secret`];
  let timeLockContractId = process.env.timelock_self_managed_contract_id;

  const keyPair = Keypair.fromSecret(timeLockOwnerSecret);
  let params = [nativeToScVal(newOwner, { type: "address" })];
  await invokeContract(keyPair, timeLockContractId, "update_owner", params);

  const newOwnerKeyPair = Keypair.fromSecret(newOwnerSecret);
  let oldOwner = process.env[`${timeLockOwnerLabel}_pubkey`];
  params = [nativeToScVal(oldOwner, { type: "address" })];
  await invokeContract(newOwnerKeyPair, timeLockContractId, "update_owner", params);
}

async function main(newOwner, newOwnerSecret) {
  await scheduleAndExecuteUpdateOwner(newOwner, newOwnerSecret);
  await updateOwnerDirectly(newOwner, newOwnerSecret);
}

module.exports = {
  updateTimeLockOwner: main,
};
