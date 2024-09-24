# Stream CLI

🚧 **Alpha Testing Notice**

This project is currently in **alpha testing**. It may contain bugs, and features are subject to change. Use caution when running this program, and do not use it with accounts holding significant funds.

---

Stream CLI is a command-line interface tool designed to automate the execution of crypto coin trades based on your strategy. It allows you to schedule trades over a 24-hour period, executing them at random times within specified windows, and adhering to user-defined parameters such as daily amount, number of trades, and minimum acceptable price.

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
  - [Initial Setup](#initial-setup)
- [How It Works](#how-it-works)
- [Transaction History](#transaction-history)
- [Terminating the Program](#terminating-the-program)
- [Limitations](#limitations)
- [Environments](#environments)
- [Warnings](#warnings)
- [Contributions](#contributions)
- [License](#license)

## Features

- **Automated Trading**: Schedule and execute crypto trades based on your strategy.
- **Account Balance Query**: Easily query the balances of an account given its address.
- **Customizable Parameters**: Define daily amounts, number of trades, and minimum prices.
- **Randomized Execution Times**: Trades occur at random times within defined windows.
- **Osmosis Network Integration**: Currently supports the Osmosis blockchain network.
- **Detailed Transaction Logging**: Stores comprehensive transaction details for auditing.

## Installation

To install and run Stream CLI, follow these steps:

1. **Install dependencies**.

```bash
sudo apt install build-essential -y
sudo apt-get install pkg-config -y
sudo apt-get install libssl-dev -y
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

2. **Clone the repository**.

```bash
git clone https://github.com/soaresa/stream-cli.git
```

3. **Navigate to the repository root directory**.

```bash
cd stream-cli
```

4. **Build the program**.

```bash
cargo build
```

## Usage

Stream CLI provides two main commands:

- **`stream`**: Automate your trading strategy by scheduling trades.
- **`balance`**: Query the balances of an account given its address.

### **Stream Command**

To start the streaming trades, use the `stream` subcommand with the appropriate options. You can choose to specify either the amount of tokens you want to **obtain** (via `--daily-amount-out`) or the amount of tokens you want to **sell** (via `--daily-amount-in`).

#### Options:

- `--daily-amount-out`: The total amount of tokens you wish to **obtain** per day.
- `--daily-amount-in`: The total amount of tokens you wish to **sell** per day.
- `--daily-streams`: The number of trades to be executed over 24 hours.
- `--min-price`: The minimum price you are willing to pay per token.

#### Examples:

- **Obtain 20 tokens over 4 trades per day** at a minimum price of 0.1 per token:

```bash
cargo run -- stream --daily-amount-out 20 --daily-streams 4 --min-price 0.1
```

- **Sell 1000 tokens over 4 trades per day**, with a target minimum price of 0.1 per token:

```bash
cargo run -- stream --daily-amount-in 1000 --daily-streams 4 --min-price 0.1
```

In these examples, the program will either aim to **obtain** 20 tokens throughout the day or **sell** 1000 tokens, executing trades across 4 intervals, depending on which option is provided (`amount-in` or `amount-out`).

### **Balance Command**

To query the balances of an account, use the `balance` subcommand:

```bash
cargo run -- balance --address <your_account_address>
```

- `--address`: The account address to query.

**Example:**

```bash
cargo run -- balance --address osmo1youraddresshere
```

### Initial Setup

1. **Enter Your Mnemonic:**

   Upon starting, the program will prompt you to enter your mnemonic (seed phrase). This is required to access your account for executing trades.

2. **Confirm Parameters:**

   The program will display:

   - The parameters you have provided.
   - The account address corresponding to your mnemonic.

   Confirm the details by typing `y` to start the program.

## How It Works

- **Time Division:**

  - The program divides the 24-hour period into the specified number of trade windows (e.g., 4 windows for 4 trades).
  - For each window, it selects a random time to execute the trade.

- **Trade Execution:**

  - At the scheduled time, the program checks:
    - If the current price of the input token is greater than or equal to the user-defined minimum price.
    - If your account has sufficient balance to execute the trade.

- **Retry Mechanism:**

  - If the conditions are not met, the program retries every 5 seconds until the end of the current window.
  - If the trade cannot be executed within the window, it is skipped.
  - A new window begins with a new random trade time.

## Transaction History

- **Storage Location:**

  - Transactions are stored in `~/stream/test/osmosis_transactions.json` or `~/stream/prod/osmosis_transactions.json`, depending on the environment.

- **Transaction Details:**

  Each transaction record includes:

  - `txhash`
  - `timestamp`
  - `pool_id`
  - `token_in`
  - `token_out`
  - `amount_out`
  - `min_price`
  - `tx_status` (broadcasted, executed, error, timeout)
  - `status_code`
  - `raw_log`
  - `gas_used`
  - `tokens_in`

- **Transaction Statuses:**

  - **broadcasted**: Transaction has been sent.
  - **executed**: Transaction was successful (`code` is 0).
  - **error**: Transaction failed (`code` is non-zero).
  - **timeout**: No response received within 60 seconds.

## Terminating the Program

To stop the program gracefully, press `Ctrl + C`. This will ensure the program completes its current operations before shutting down, preventing any unexpected interruptions or data loss.

## Limitations

- **Network Support**: Currently, Stream CLI only integrates with the Osmosis network.
- **Configuration**: Pool IDs and tokens for each environment are defined in the `constants.rs` module.

## Environments

- **Setting the Environment:**

  - Modify the `ENVIRONMENT` variable in the `.env` file at the root of the project.
  - Possible values:
    - `prod`: Uses the `osmosis-1` chain.
    - `test`: Uses the `osmo-test-5` chain.

## Warnings

- **Experimental Software**:

  - This repository is an experiment and may contain bugs.
  - Use caution when running this program.

- **Security Notice**:

  - You will need to provide your mnemonic (seed phrase).
  - Ensure you are in a secure environment to prevent compromising your account.

- **Responsibility**:

  - You are responsible for any actions taken by this program.
  - Use at your own risk.

## Contributions

If you encounter any bugs or have suggestions for architectural or security improvements, please:

- **Open an Issue**: Describe the problem or enhancement in detail.
- **Submit a Pull Request**: Contribute code directly to the repository.

---

By following the instructions above, you can automate your crypto trading strategy using Stream CLI. Remember to use this tool responsibly and ensure that you understand the risks involved with automated trading.
