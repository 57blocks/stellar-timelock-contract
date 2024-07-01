const {
  Networks,
  TransactionBuilder,
  Operation,
  BASE_FEE,
  scValToNative,
  SorobanRpc,
} = require("@stellar/stellar-sdk");

const HorizonUrl = "http://localhost:8000";
const RpcUrl = `${HorizonUrl}/soroban/rpc`;

const rpc = new SorobanRpc.Server(RpcUrl, { allowHttp: true });

async function scheduleOperation(keyPair, timeLockContractId, scheduleParams) {
  const account = await rpc.getAccount(keyPair.publicKey());
  const scheduleTransaction = new TransactionBuilder(account, {
    fee: BASE_FEE,
    networkPassphrase: Networks.STANDALONE,
  })
    .addOperation(
      Operation.invokeContractFunction({
        contract: timeLockContractId,
        function: "schedule",
        args: scheduleParams,
      })
    )
    .setTimeout(0)
    .build();

  let tx;
  let simRes = await rpc.simulateTransaction(scheduleTransaction);

  if (SorobanRpc.Api.isSimulationSuccess(simRes)) {
    tx = SorobanRpc.assembleTransaction(scheduleTransaction, simRes).build();
  } else {
    console.log(await rpc._simulateTransaction(scheduleTransaction));
    throw new Error("Schedule operation failed to simulate");
  }

  tx.sign(keyPair);
  const sendRes = await rpc.sendTransaction(tx);
  console.log(`sendResponse hash = ${sendRes.hash}`);

  if (sendRes.status === "PENDING") {
    await new Promise((resolve) => setTimeout(resolve, 5000));
    const getRes = await rpc.getTransaction(sendRes.hash);

    // console.log("getRes", getRes);
    if (getRes.status !== "NOT_FOUND") {
      console.log(`Schedule operation: ${getRes.status}`);
      if (getRes.status === "SUCCESS") {
        let txMeta = getRes.resultMetaXdr;
        let returnValue = txMeta.v3().sorobanMeta().returnValue();
        let nativeValue = scValToNative(returnValue);
        console.log(`operationId = ${nativeValue.toString("hex")}`);
        return {result: true, operationId: nativeValue.toString("hex")};
      }
    } else console.log(await rpc._getTransaction(sendRes.hash));
  } else {
    console.log(await rpc._sendTransaction(tx));
  }
  return {result: false};
}

async function executeOperation(keyPair, timeLockContractId, executeParams) {
  const account = await rpc.getAccount(keyPair.publicKey());
  const txBuilder = new TransactionBuilder(account, {
    fee: BASE_FEE,
    networkPassphrase: Networks.STANDALONE,
  });
  let executeTransaction = txBuilder
    .addOperation(
      Operation.invokeContractFunction({
        contract: timeLockContractId,
        function: "execute",
        args: executeParams,
      })
    )
    .setTimeout(0)
    .build();

  let tx;
  let simRes = await rpc.simulateTransaction(executeTransaction);
  if (SorobanRpc.Api.isSimulationSuccess(simRes)) {
    tx = SorobanRpc.assembleTransaction(executeTransaction, simRes).build();
  } else {
    console.log(await rpc._simulateTransaction(executeTransaction));
    throw new Error("Execute operation failed to simulate");
  }

  tx.sign(keyPair);
  const sendRes = await rpc.sendTransaction(tx);
  console.log(`sendResponse hash = ${sendRes.hash}`);

  if (sendRes.status === "PENDING") {
    await new Promise((resolve) => setTimeout(resolve, 5000));
    const getRes = await rpc.getTransaction(sendRes.hash);

    // console.log("getRes", getRes);
    if (getRes.status !== "NOT_FOUND") {
      console.log(`Execute operation:  ${getRes.status}`);
    } else console.log(await rpc._getTransaction(sendRes.hash));
  } else {
    console.log(await rpc._sendTransaction(tx));
  }
}

async function invokeContract(keyPair, contractId, functionName, args) {
  const account = await rpc.getAccount(keyPair.publicKey());
  const transaction = new TransactionBuilder(account, {
    fee: BASE_FEE,
    networkPassphrase: Networks.STANDALONE,
  })
    .addOperation(
      Operation.invokeContractFunction({
        contract: contractId,
        function: functionName,
        args: args,
      })
    )
    .setTimeout(0)
    .build();

  let tx;
  let simRes = await rpc.simulateTransaction(transaction);
  if (SorobanRpc.Api.isSimulationSuccess(simRes)) {
    tx = SorobanRpc.assembleTransaction(transaction, simRes).build();
  } else {
    console.log(await rpc._simulateTransaction(transaction));
    throw new Error(`Invoke contract ${functionName} failed to simulate`);
  }

  tx.sign(keyPair);
  const sendRes = await rpc.sendTransaction(tx);
  console.log(`sendResponse hash = ${sendRes.hash}`);

  if (sendRes.status === "PENDING") {
    await new Promise((resolve) => setTimeout(resolve, 5000));
    const getRes = await rpc.getTransaction(sendRes.hash);

    if (getRes.status !== "NOT_FOUND") {
      console.log(`Invoke contract ${functionName} :  ${getRes.status}`);
      if (getRes.status === "SUCCESS") {
        let txMeta = getRes.resultMetaXdr;
        let returnValue = txMeta.v3().sorobanMeta().returnValue();
        let nativeValue = scValToNative(returnValue);
        return nativeValue;
      }
    } else console.log(await rpc._getTransaction(sendRes.hash));
  } else {
    console.log(await rpc._sendTransaction(tx));
  }
}

module.exports = {
  HorizonUrl,
  RpcUrl,
  scheduleOperation,
  executeOperation,
  invokeContract,
};
