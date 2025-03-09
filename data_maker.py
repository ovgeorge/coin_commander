import yaml
import os

# Define sample data
signers_data = [
    {
        "name": "Signer1",
        "accounts": [
            {
                "name": "Account1",
                "chains": [
                    {
                        "name": "Ethereum",
                        "assets": [
                            {"name": "ETH", "amount": 5.2},
                            {"name": "USDT", "amount": 1000.0}
                        ]
                    },
                    {
                        "name": "Polygon",
                        "assets": [
                            {"name": "MATIC", "amount": 150.5},
                            {"name": "USDC", "amount": 500.0}
                        ]
                    }
                ]
            },
            {
                "name": "Account2",
                "chains": [
                    {
                        "name": "Binance",
                        "assets": [
                            {"name": "BNB", "amount": 10.0},
                            {"name": "BUSD", "amount": 2000.0}
                        ]
                    }
                ]
            }
        ]
    },
    {
        "name": "Signer2",
        "accounts": [
            {
                "name": "Account3",
                "chains": [
                    {
                        "name": "Solana",
                        "assets": [
                            {"name": "SOL", "amount": 75.3},
                            {"name": "SRM", "amount": 300.0}
                        ]
                    }
                ]
            }
        ]
    }
]

# Create directory structure and files
def create_data_files():
    # Create base directory
    base_dir = "wallet_data"
    os.makedirs(base_dir, exist_ok=True)

    # Create signers file
    signers_file = os.path.join(base_dir, "signers.yaml")
    with open(signers_file, 'w') as f:
        yaml.dump({"signers": [s["name"] for s in signers_data]}, f)

    # For each signer
    for signer in signers_data:
        signer_dir = os.path.join(base_dir, signer["name"])
        os.makedirs(signer_dir, exist_ok=True)

        # Create accounts file for this signer
        accounts_file = os.path.join(signer_dir, "accounts.yaml")
        with open(accounts_file, 'w') as f:
            yaml.dump({"accounts": [a["name"] for a in signer["accounts"]]}, f)

        # For each account
        for account in signer["accounts"]:
            account_dir = os.path.join(signer_dir, account["name"])
            os.makedirs(account_dir, exist_ok=True)

            # Create chains file for this account
            chains_file = os.path.join(account_dir, "chains.yaml")
            with open(chains_file, 'w') as f:
                yaml.dump({"chains": [c["name"] for c in account["chains"]]}, f)

            # For each chain
            for chain in account["chains"]:
                chain_file = os.path.join(account_dir, f"{chain['name']}.yaml")
                with open(chain_file, 'w') as f:
                    yaml.dump(chain, f)

if __name__ == "__main__":
    create_data_files()
    print("Data files created in 'wallet_data' directory")
