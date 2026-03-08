# ChainLens 🔍

ChainLens is a **Bitcoin blockchain analysis CLI tool** that parses raw Bitcoin block data and generates structured reports about blocks, transactions, scripts, and potential warnings.



---

## Features

* Parses raw Bitcoin block data
* Analyzes transactions (inputs, outputs, scripts)
* Generates structured reports for:

  * Blocks
  * Transactions
  * Inputs and Outputs
  * Script types
* Detects potential warnings in transactions
* Supports analysis of compressed block fixture files

---

## Project Structure

```
Chain-lens
│
├── src/            # Core blockchain parsing and analysis logic
├── fixtures/       # Sample Bitcoin block data used for testing
├── static/         # Static resources for reports
├── grader/         # Evaluation scripts
│
├── cli.sh          # CLI entrypoint
├── setup.sh        # Project setup script
├── web.sh          # Web visualization script
├── grade.sh        # Automated grading script
│
├── Cargo.toml      # Rust dependencies and configuration
└── README.md
```

---

## Tech Stack

* **Rust**
* Bitcoin block parsing
* CLI scripting (Bash)
* Data analysis and reporting

---

## Installation

Clone the repository:

```
git clone https://github.com/sarvesh227/Chain-lens.git
cd Chain-lens
```

Run setup:

```
chmod +x setup.sh
./setup.sh
```

---

## Usage

Run the CLI tool:

```
./cli.sh
```

Run the grading tests:

```
./grade.sh
```

---

## What I Learned

* Parsing raw Bitcoin block data
* Understanding transaction structures (UTXO model)
* Working with compressed blockchain datasets
* Building CLI tools in Rust
* Structuring a production-style repository

---

## Acknowledgement

This project was inspired by the **Summer of Bitcoin Developer Challenge**, which focuses on helping developers understand the Bitcoin ecosystem and blockchain internals.

---

## Author

Sarvesh Goel
GitHub: https://github.com/sarvesh227
