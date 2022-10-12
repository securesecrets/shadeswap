# ShadeSwap Core Contracts
| Contract                    | Reference                         | Description                           |
| --------------------------- | --------------------------------- | ------------------------------------- |
| [`amm_pair`](./contracts/amm_pair)  | [doc](./contracts/amm_pair/README.md) | |
| [`factory`](./contracts/factory)  | [doc](./contracts/factory/README.md) |  |
| [`lp_token`](./contracts/lp_token)  | [doc](./contracts/lp_token/README.md) |  |
| [`router`](./contracts/router)  | [doc](./contracts/router/README.md) |  |
| [`snip20`](./contracts/snip20)  | [doc](./contracts/snip20/README.md) |  |
| [`staking`](./contracts/staking)  | [doc](./contracts/staking/README.md) |  |

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
