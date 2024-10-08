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

async function scheduleAndExecuteUpdateMinDelay() {
  let timeLockContractId = process.env.timelock_contract_id;
  let proposerLabel = users.proposer;
  let proposerSecret = process.env[`${proposerLabel}_secret`];
  let proposer = process.env[`${proposerLabel}_pubkey`];
  let target = process.env.timelock_contract_id;
  let fnName = "update_min_delay";
  let newMinDelay = 5;
  let data = [nativeToScVal(newMinDelay, { type: "u64" })];
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
  const {result, operationId} = await scheduleOperation(
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
    let srValOperationId = nativeToScVal(Buffer.from(operationId, "hex"), { type: "bytes" });
    let timelock = await invokeContract(keyPair, timeLockContractId, "get_schedule_lock_time", [srValOperationId]);
    console.log(`before execute timelock: ${timelock}`);
    await executeOperation(executorKeyPair, timeLockContractId, executeParams);
    timelock = await invokeContract(keyPair, timeLockContractId, "get_schedule_lock_time", [srValOperationId]);
    console.log(`after execute timelock: ${timelock}`);
  }
}

async function updateMinDelayDirectly() {
  let timeLockOwnerLabel = users.timelockOwner;
  let newMinDelay = 5;
  let timeLockOwnerSecret = process.env[`${timeLockOwnerLabel}_secret`];
  let timeLockContractId = process.env.timelock_self_managed_contract_id;

  let params = [nativeToScVal(newMinDelay, { type: "u64" })];

  const keyPair = Keypair.fromSecret(timeLockOwnerSecret);

  await invokeContract(keyPair, timeLockContractId, "update_min_delay", params);
}

async function main() {
  await scheduleAndExecuteUpdateMinDelay();
  await updateMinDelayDirectly();
}

module.exports = {
  updateTimeLockMinDelay: main,
};
