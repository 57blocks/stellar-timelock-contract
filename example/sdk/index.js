require("dotenv").config();

const { generateNewKeypair, users } = require("../common");
const { tokenMint } = require("./token-mint");
const { updateTokenAdmin } = require("./token-updateAdmin");
const { updateTimeLockMinDelay } = require("./timelock-updateMinDelay");
const { addRoleToTimeLock } = require("./timelock-grantRole");
const { revokeRoleFromTimeLock } = require("./timelock-revokeRole");
const { updateTimeLockOwner } = require("./timelock-updateOwner");

async function main() {
  console.log("Mock token minting...")
  await tokenMint();
  console.log("Mock token update admin...");
  await updateTokenAdmin();
  console.log("Time lock update min delay...");
  await updateTimeLockMinDelay();
  console.log("Time lock grant role...");
  const roleAccount = generateNewKeypair();
  const role = 3;
  await addRoleToTimeLock(roleAccount, role);
  console.log("Time lock revoke role...");
  await revokeRoleFromTimeLock(roleAccount, role);
  console.log("Time lock update owner...");
  let newOwnerLabel = users.deployer;
  let newOwner = process.env[`${newOwnerLabel}_pubkey`];
  let newOwnerSecret = process.env[`${newOwnerLabel}_secret`];
  console.log(`new owner public key: ${newOwner}`);
  console.log(`new owner secret key: ${newOwnerSecret}`);
  await updateTimeLockOwner(newOwner, newOwnerSecret);
}

main()
  .then(() => {
    console.log("✅");
  })
  .catch((e) => {
    console.error(e);
  });
