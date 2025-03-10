use cursive::view::{Nameable, Scrollable};
use cursive::{
    traits::*,
    views::{Dialog, LinearLayout, Panel, SelectView, TextView},
    Cursive,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Asset {
    name: String,
    amount: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Chain {
    name: String,
    assets: Vec<Asset>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Account {
    name: String,
    chains: Vec<Chain>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Signer {
    name: String,
    accounts: Vec<Account>,
}

#[derive(Debug, Serialize, Deserialize)]
enum SelectionState {
    SignerLevel { signers: Vec<Signer>, selected: Option<usize> },
    AccountLevel { signers: Vec<Signer>, signer_idx: usize, selected: Option<usize> },
    ChainLevel { signers: Vec<Signer>, signer_idx: usize, account_idx: usize, selected: Option<usize> },
    AssetLevel { signers: Vec<Signer>, signer_idx: usize, account_idx: usize, chain_idx: usize, selected: Option<usize> },
}

#[derive(Debug, Serialize, Deserialize)]
struct SystemState {
    state: SelectionState,
}

#[derive(Debug, Serialize, Deserialize)]
struct WalletData {
    signers: Vec<Signer>,
}

fn load_system_state() -> Result<SystemState, Box<dyn std::error::Error>> {
    let wallet_yaml = fs::read_to_string("wallet_data.yaml")?;
    let wallet_data: WalletData = serde_yaml::from_str(&wallet_yaml)?;

    let mut file = fs::File::create("coins.txt")?;
    for signer in &wallet_data.signers {
        for account in &signer.accounts {
            for chain in &account.chains {
                writeln!(file, "Signer: {}", signer.name)?;
                writeln!(file, "Account: {}", account.name)?;
                writeln!(file, "Chain: {}", chain.name)?;
                for asset in &chain.assets {
                    writeln!(file, "  {}: {}", asset.name, asset.amount)?;
                }
            }
        }
    }

    Ok(SystemState {
        state: SelectionState::SignerLevel {
            signers: wallet_data.signers,
            selected: None,
        },
    })
}

fn main() {
    let mut siv = cursive::default();

    let state = load_system_state().expect("Failed to load system state");
    siv.set_user_data(state);

    let signers = match &siv.user_data::<SystemState>().unwrap().state {
        SelectionState::SignerLevel { signers, .. } => signers.clone(),
        _ => panic!("Unexpected initial state"),
    };

    let signer_select = SelectView::<usize>::new()
        .on_select(move |s, &signer_idx| {
            let state = s.user_data::<SystemState>().unwrap();
            let signers = match &state.state {
                SelectionState::SignerLevel { signers, .. } => signers.clone(),
                SelectionState::AccountLevel { signers, .. } => signers.clone(),
                SelectionState::ChainLevel { signers, .. } => signers.clone(),
                SelectionState::AssetLevel { signers, .. } => signers.clone(),
            };
            let accounts = signers[signer_idx].accounts.clone();

            state.state = SelectionState::AccountLevel {
                signers,
                signer_idx,
                selected: None,
            };

            s.with(|siv| {
                siv.call_on_name("account_select", |view: &mut SelectView<usize>| {
                    view.clear();
                    for (idx, account) in accounts.iter().enumerate() {
                        view.add_item(account.name.clone(), idx);
                    }
                });
                siv.call_on_name("chain_select", |view: &mut SelectView<usize>| {
                    view.clear();
                });
                siv.call_on_name("coin_select", |view: &mut SelectView<usize>| {
                    view.clear();
                });
            });
        })
        .with(|view| {
            for (idx, signer) in signers.iter().enumerate() {
                view.add_item(signer.name.clone(), idx);
            }
        })
        .with_name("signer_select")
        .scrollable()
        .fixed_width(10);

    let account_select = SelectView::<usize>::new()
        .on_select(move |s, &account_idx| {
            let state = s.user_data::<SystemState>().unwrap();
            let (signers, signer_idx) = match &state.state {
                SelectionState::SignerLevel { signers, .. } => (signers.clone(), 0),
                SelectionState::AccountLevel { signers, signer_idx, .. } => (signers.clone(), *signer_idx),
                SelectionState::ChainLevel { signers, signer_idx, .. } => (signers.clone(), *signer_idx),
                SelectionState::AssetLevel { signers, signer_idx, .. } => (signers.clone(), *signer_idx),
            };
            let chains = signers[signer_idx].accounts[account_idx].chains.clone();

            state.state = SelectionState::ChainLevel {
                signers: signers.clone(),
                signer_idx,
                account_idx,
                selected: None,
            };

            s.with(|siv| {
                siv.call_on_name("chain_select", |view: &mut SelectView<usize>| {
                    view.clear();
                    for (idx, chain) in chains.iter().enumerate() {
                        view.add_item(chain.name.clone(), idx);
                    }
                });
                siv.call_on_name("coin_select", |view: &mut SelectView<usize>| {
                    view.clear();
                });
            });
        })
        .with_name("account_select")
        .scrollable()
        .fixed_width(20);

    let chain_select = SelectView::<usize>::new()
        .on_select(move |s, &chain_idx| {
            let state = s.user_data::<SystemState>().unwrap();
            let (signers, signer_idx, account_idx) = match &state.state {
                SelectionState::SignerLevel { signers, .. } => (signers.clone(), 0, 0),
                SelectionState::AccountLevel { signers, signer_idx, .. } => (signers.clone(), *signer_idx, 0),
                SelectionState::ChainLevel { signers, signer_idx, account_idx, .. } => (signers.clone(), *signer_idx, *account_idx),
                SelectionState::AssetLevel { signers, signer_idx, account_idx, .. } => (signers.clone(), *signer_idx, *account_idx),
            };
            let assets = signers[signer_idx].accounts[account_idx].chains[chain_idx].assets.clone();

            state.state = SelectionState::AssetLevel {
                signers: signers.clone(),
                signer_idx,
                account_idx,
                chain_idx,
                selected: None,
            };

            s.with(|siv| {
                siv.call_on_name("coin_select", |view: &mut SelectView<usize>| {
                    view.clear();
                    for (idx, asset) in assets.iter().enumerate() {
                        let item = format!("{}: {}", asset.name, asset.amount);
                        view.add_item(item, idx);
                    }
                });
            });
        })
        .with_name("chain_select")
        .scrollable()
        .fixed_width(20);

    let coin_select = SelectView::<usize>::new()
        .on_select(|s, &asset_idx| {
            let state = s.user_data::<SystemState>().unwrap();
            if let SelectionState::AssetLevel { selected, .. } = &mut state.state {
                *selected = Some(asset_idx);
                // Optional feedback
                // s.add_layer(Dialog::info(format!("Selected asset at index {}", asset_idx)));
            }
        })
        .with_name("coin_select")
        .scrollable()
        .fixed_width(20);

    let signer_pane = LinearLayout::vertical()
        .child(TextView::new("Signers").center())
        .child(signer_select);

    let account_pane = LinearLayout::vertical()
        .child(TextView::new("Accounts").center())
        .child(account_select);

    let chain_pane = LinearLayout::vertical()
        .child(TextView::new("Chains").center())
        .child(chain_select);

    let coin_pane = LinearLayout::vertical()
        .child(TextView::new("Coins").center())
        .child(coin_select);

    let content = LinearLayout::horizontal()
        .child(Panel::new(signer_pane).title("Signers"))
        .child(Panel::new(account_pane).title("Accounts"))
        .child(Panel::new(chain_pane).title("Chains"))
        .child(Panel::new(coin_pane).title("Coins"));

    siv.add_layer(
        Dialog::new()
            .title("Signer, Account, Chain, and Coin Selector")
            .content(content),
    );

    siv.run();
}
