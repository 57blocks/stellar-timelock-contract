const { Networks, TransactionBuilder, Operation, BASE_FEE, scValToNative, rpc } = require('@stellar/stellar-sdk');

const HorizonUrl = 'http://localhost:8000';
const RpcUrl = `${HorizonUrl}/soroban/rpcServer`;

const rpcServer = new rpc.Server(RpcUrl, { allowHttp: true });

async function scheduleOperation(keyPair, timeLockContractId, scheduleParams) {
  const account = await rpcServer.getAccount(keyPair.publicKey());
  const scheduleTransaction = new TransactionBuilder(account, {
    fee: BASE_FEE,
    networkPassphrase: Networks.STANDALONE,
  })
    .addOperation(
      Operation.invokeContractFunction({
        contract: timeLockContractId,
        function: 'schedule',
        args: scheduleParams,
      })
    )
    .setTimeout(0)
    .build();

  let tx;
  let simRes = await rpcServer.simulateTransaction(scheduleTransaction);

  if (rpc.Api.isSimulationSuccess(simRes)) {
    tx = rpc.assembleTransaction(scheduleTransaction, simRes).build();
  } else {
    console.log(await rpcServer._simulateTransaction(scheduleTransaction));
    throw new Error('Schedule operation failed to simulate');
  }

  tx.sign(keyPair);
  const sendRes = await rpcServer.sendTransaction(tx);
  console.log(`sendResponse hash = ${sendRes.hash}`);

  if (sendRes.status === 'PENDING') {
    await new Promise((resolve) => setTimeout(resolve, 5000));
    const getRes = await rpcServer.getTransaction(sendRes.hash);

    // console.log("getRes", getRes);
    if (getRes.status !== 'NOT_FOUND') {
      console.log(`Schedule operation: ${getRes.status}`);
      if (getRes.status === 'SUCCESS') {
        let txMeta = getRes.resultMetaXdr;
        let returnValue = txMeta.v3().sorobanMeta().returnValue();
        let nativeValue = scValToNative(returnValue);
        console.log(`operationId = ${nativeValue.toString('hex')}`);
        return { result: true, operationId: nativeValue.toString('hex') };
      }
    } else console.log(await rpcServer._getTransaction(sendRes.hash));
  } else {
    console.log(await rpcServer._sendTransaction(tx));
  }
  return { result: false };
}

async function executeOperation(keyPair, timeLockContractId, executeParams) {
  const account = await rpcServer.getAccount(keyPair.publicKey());
  const txBuilder = new TransactionBuilder(account, {
    fee: BASE_FEE,
    networkPassphrase: Networks.STANDALONE,
  });
  let executeTransaction = txBuilder
    .addOperation(
      Operation.invokeContractFunction({
        contract: timeLockContractId,
        function: 'execute',
        args: executeParams,
      })
    )
    .setTimeout(0)
    .build();

  let tx;
  let simRes = await rpcServer.simulateTransaction(executeTransaction);
  if (rpc.Api.isSimulationSuccess(simRes)) {
    tx = rpc.assembleTransaction(executeTransaction, simRes).build();
  } else {
    console.log(await rpcServer._simulateTransaction(executeTransaction));
    throw new Error('Execute operation failed to simulate');
  }

  tx.sign(keyPair);
  const sendRes = await rpcServer.sendTransaction(tx);
  console.log(`sendResponse hash = ${sendRes.hash}`);

  if (sendRes.status === 'PENDING') {
    await new Promise((resolve) => setTimeout(resolve, 5000));
    const getRes = await rpcServer.getTransaction(sendRes.hash);

    // console.log("getRes", getRes);
    if (getRes.status !== 'NOT_FOUND') {
      console.log(`Execute operation:  ${getRes.status}`);
    } else console.log(await rpcServer._getTransaction(sendRes.hash));
  } else {
    console.log(await rpcServer._sendTransaction(tx));
  }
}

async function invokeContract(keyPair, contractId, functionName, args) {
  const account = await rpcServer.getAccount(keyPair.publicKey());
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
  let simRes = await rpcServer.simulateTransaction(transaction);
  if (rpc.Api.isSimulationSuccess(simRes)) {
    tx = rpc.assembleTransaction(transaction, simRes).build();
  } else {
    console.log(await rpcServer._simulateTransaction(transaction));
    throw new Error(`Invoke contract ${functionName} failed to simulate`);
  }

  tx.sign(keyPair);
  const sendRes = await rpcServer.sendTransaction(tx);
  console.log(`sendResponse hash = ${sendRes.hash}`);

  if (sendRes.status === 'PENDING') {
    await new Promise((resolve) => setTimeout(resolve, 5000));
    const getRes = await rpcServer.getTransaction(sendRes.hash);

    if (getRes.status !== 'NOT_FOUND') {
      console.log(`Invoke contract ${functionName} :  ${getRes.status}`);
      if (getRes.status === 'SUCCESS') {
        let txMeta = getRes.resultMetaXdr;
        let returnValue = txMeta.v3().sorobanMeta().returnValue();
        let nativeValue = scValToNative(returnValue);
        return nativeValue;
      }
    } else console.log(await rpcServer._getTransaction(sendRes.hash));
  } else {
    console.log(await rpcServer._sendTransaction(tx));
  }
}

module.exports = {
  HorizonUrl,
  RpcUrl,
  scheduleOperation,
  executeOperation,
  invokeContract,
};
