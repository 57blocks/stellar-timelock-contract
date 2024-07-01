require("dotenv").config();

const { nativeToScVal, Address, Keypair } = require("@stellar/stellar-sdk");

const { users, minDelay, sleep, generateNewKeypair } = require("../common");
const {
  scheduleOperation,
  executeOperation,
  invokeContract,
} = require("./common");

async function scheduleAndExecuteUpdateAdmin() {
  const newAdmin = process.env[`${users.deployer}_pubkey`];
  const newAdminSecret = process.env[`${users.deployer}_secret`];
  let timeLockContractId = process.env.timelock_contract_id;
  let proposerLabel = users.proposer;
  let proposerSecret = process.env[`${proposerLabel}_secret`];
  let proposer = process.env[`${proposerLabel}_pubkey`];
  let target = process.env.mock_token_contract_id;
  let fnName = "set_admin";
  let data = [nativeToScVal(newAdmin, { type: "address" })];
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
    await executeOperation(executorKeyPair, timeLockContractId, executeParams);
  }

  let tokenAdminKeypair = Keypair.fromSecret(newAdminSecret);
  let user = generateNewKeypair();
  let mintParams = [
    nativeToScVal(user, { type: "address" }),
    nativeToScVal(1000000000, { type: "i128" }),
  ];

  let balance = await invokeContract(tokenAdminKeypair, target, "balance", [
    nativeToScVal(user, { type: "address" }),
  ]);
  console.log("before mint balance: ", balance);
  await invokeContract(tokenAdminKeypair, target, "mint", mintParams);
  balance = await invokeContract(tokenAdminKeypair, target, "balance", [
    nativeToScVal(user, { type: "address" }),
  ]);
  console.log("after mint balance: ", balance);

  // rollback to the original admin
  await invokeContract(tokenAdminKeypair, target, "set_admin", [
    nativeToScVal(timeLockContractId, { type: "address" }),
  ]);
}

async function main() {
  await scheduleAndExecuteUpdateAdmin();
}

module.exports = {
  updateTokenAdmin: main,
};
