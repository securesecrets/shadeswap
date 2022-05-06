# ShadeSwap Core Contracts
| Contract                    | Reference                         | Description                           |
| --------------------------- | --------------------------------- | ------------------------------------- |
| [`amm_pair`](./contracts/governance)  | [doc](./contracts/amm_pair/README.md) | |
| [`factory`](./contracts/staking)  | [doc](./contracts/factory/README.md) |  |
| [`router`](./contracts/scrt_staking)  | [doc](./contracts/router/README.md) |  |

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
