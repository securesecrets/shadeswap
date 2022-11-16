# ShadeSwap Core Contracts
| Contract                    | Reference                         | Description                           |
| --------------------------- | --------------------------------- | ------------------------------------- |
| [`amm_pair`](./contracts/amm_pair)  | [doc](./contracts/amm_pair/README.md) | Individual contract used to manage a single LP pool. The contract can inherit some settings from Factory such as custom Fee.|
| [`factory`](./contracts/factory)  | [doc](./contracts/factory/README.md) |Factory Contract that can be used to centrally manage pair contracts after initialization|
| [`lp_token`](./contracts/lp_token)  | [doc](./contracts/lp_token/README.md) |LP Token given to users after they have added liquidity to a amm_pair contract|
| [`router`](./contracts/router)  | [doc](./contracts/router/README.md) |Router contract used to allow for multi-hop trades|
| [`snip20`](./contracts/snip20)  | [doc](./contracts/snip20/README.md) |Snip20 reference implementation used for testing|
| [`staking`](./contracts/staking)  | [doc](./contracts/staking/README.md) |Staking contract that allows for users to gain rewards from adding liquidity|

## Development Environment

### Environment Setup

1. Make sure [Docker](https://www.docker.com/) is installed

2. Pull the SN-testnet image
```shell
make server-download
```

3. Open a terminal inside this repo and run:
```shell
make server-start
```

4. Inside another terminal run:
```shell
make server-connect
```

#### Testing the environment
Inside the container, go to /root/code and compile all the smart contracts:
```
make
```
Then test run all the Protocol unit-tests and integration tests using the [tester](packages/network_integration):
```shell
make integration-tests
```

### Unit Tests

Each contract contains Rust unit and integration tests embedded within the contract source directories. You can run:

```sh
cargo unit-test
```

### CLI 

For CLI please download the latest secretcli and add to your path if Linux
```
export PATH=/to/your/secretcli_folder:$PATH

WSL for linux Example
export PATH=/mnt/d/secretcli:$PATH
```
