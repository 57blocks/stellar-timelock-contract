require("dotenv").config();

const { cmd, networks, users } = require("../common");

let tokenContractId = process.env.mock_token_contract_id;

async function getTokenBalance(pubKey) {
    let balance = await cmd(`soroban contract invoke --id ${tokenContractId} --source ${users.deployer} --network ${networks.name} -- balance --id ${pubKey}`);
    console.log(`${pubKey}'s balance: ${balance}`);
    return balance;
}

async function tokenMint(caller,to, amount) {
    await cmd(`soroban contract invoke --id ${tokenContractId} --source ${caller} --network ${networks.name} -- mint --to ${to} --amount ${amount}`);
}

async function updateTokenAdmin(caller, newAdmin) {
    await cmd(`soroban contract invoke --id ${tokenContractId} --source ${caller} --network ${networks.name} -- set_admin --new_admin ${newAdmin}`);
}

module.exports = {
    getTokenBalance,
    tokenMint,
    updateTokenAdmin
}