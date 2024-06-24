# Time Lock Project

## Project Structure

- /example
  client script to guide how to use time lock

- /owner
  ownable library

- /time_lock
  TimeLockController contract

- /tests
  unit cases of time lock

## Time Lock Workflow
![image](./timelock-workflow.png)

Let's briefly introduce the flow mentioned in the above diagram:

Step 1:
After deployed the Token smart contract, the deployer call initialize function with timelock as admin  to initialize the token contract.The TimeLockController's instance become the admin of the deployed Token.

Step 2:
The proposer schedule Token's mint function call by invoke TimeLockController's schedule function and get the operationId.

Step 3:
Once the locked time is over, the executor can invoke TimeLockController's execute function to execute the Token's mint.

Step 4:
If the scheduled operation doesn't need execution, the user has canceller role can cancel the operation by operationId.

## Time Lock Self Management

- initialization: After deployment, the deployer must call the `initialize` function to set up the timelock. The timelock will only become operational after this initialization. Note that this function can be invoked only once..

- update min delay time: the owner of timelock can update the min delay of every schedule operation.

- grant/revoke role: the owner of timelock can add/revoke the proposer/executor/canceller roles

- update the owner: the owner can transfer his right another one

