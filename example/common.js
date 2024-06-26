
const { spawn } = require("child_process");
const { Keypair } = require("@stellar/stellar-sdk");

exports.cmd = async (command) => {
  const internalCmd = (...command) => {
    const p = spawn(command[0], command.slice(1));
    return new Promise((resolveFunc) => {
      p.stdout.on("data", (x) => {
        resolveFunc(x.toString().replace(/\W/g, ""));
      });
      p.stderr.on("data", (x) => {
        const error = x.toString();
        if (error.startsWith("error: ")) {
          process.stderr.write(error);
        }
      });
      p.on("exit", (code) => {
        resolveFunc(code);
      });
    });
  };

  return internalCmd("bash", "-c", command);
};

exports.getRootPath = () => {
  return process.cwd().replace("/example", "");
};

exports.sleep = (ms) => {
  return new Promise((resolve) => setTimeout(resolve, ms));
};

exports.generateNewKeypair = () => {
  let keypair = Keypair.random();
  console.log(`public key: ${keypair.publicKey()}`);
  console.log(`secret key: ${keypair.secret()}`);
  return keypair.publicKey();
}

exports.users = {
  deployer: "deployer",
  timelockOwner: "lily",
  proposer: "bob",
  executor: "alice",
  cancaller: "amy",
};

exports.minDelay = 10; // 10 seconds

exports.networks = {
  name: "localnet",
};