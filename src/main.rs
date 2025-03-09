use cursive::{
    traits::*,
    views::{Dialog, LinearLayout, SelectView, TextView, Panel},
    Cursive,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
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
    SignerLevel {
        signers: Vec<Signer>,
        selected: Option<usize>,
    },
    AccountLevel {
        signers: Vec<Signer>,
        signer_idx: usize,
        selected: Option<usize>,
    },
    ChainLevel {
        signers: Vec<Signer>,
        signer_idx: usize,
        account_idx: usize,
        selected: Option<usize>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct SystemState {
    state: SelectionState,
}

#[derive(Debug, Deserialize)]
struct NameList {
    signers: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct AccountList {
    accounts: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ChainList {
    chains: Vec<String>,
}

fn load_system_state() -> Result<SystemState, Box<dyn std::error::Error>> {
    let base_path = Path::new("wallet_data");
    
    let signers_yaml = fs::read_to_string(base_path.join("signers.yaml"))?;
    let signers_list: NameList = serde_yaml::from_str(&signers_yaml)?;
    
    let mut signers = Vec::new();
    
    let mut file = std::fs::File::create(base_path.join("accounts.txt"))?;
    
    for signer_name in signers_list.signers {
        let signer_path = base_path.join(&signer_name);
        
        let accounts_yaml = fs::read_to_string(signer_path.join("accounts.yaml"))?;
        writeln!(file, "Signer: {}", signer_name)?;
        writeln!(file, "accounts.yaml content:\n{}", accounts_yaml)?;
        let accounts_list: AccountList = serde_yaml::from_str(&accounts_yaml)?;
        
        let mut accounts = Vec::new();
        
        for account_name in accounts_list.accounts {
            let account_path = signer_path.join(&account_name);
            
            let chains_yaml = fs::read_to_string(account_path.join("chains.yaml"))?;
            let chains_list: ChainList = serde_yaml::from_str(&chains_yaml)?;
            
            let mut chains = Vec::new();
            
            for chain_name in chains_list.chains {
                let chain_yaml = fs::read_to_string(account_path.join(format!("{}.yaml", chain_name)))?;
                let chain: Chain = serde_yaml::from_str(&chain_yaml)?;
                chains.push(chain);
            }
            
            accounts.push(Account {
                name: account_name,
                chains,
            });
        }
        
        signers.push(Signer {
            name: signer_name,
            accounts,
        });
    }
    
    Ok(SystemState {
        state: SelectionState::SignerLevel {
            signers,
            selected: None,
        }
    })
}

fn main() {
    let mut siv = cursive::default();

    // Load and set system state
    let state = load_system_state().expect("Failed to load system state");
    siv.set_user_data(state);

    // Extract signers from the initial state
    let signers = match &siv.user_data::<SystemState>().unwrap().state {
        SelectionState::SignerLevel { signers, .. } => signers.clone(),
        _ => panic!("Unexpected initial state"),
    };

    // Create left pane SelectView for signers
    let left_select = SelectView::<usize>::new()
    .on_select(|s, &signer_idx| {
        let state = s.user_data::<SystemState>().unwrap();
        let signers = match &state.state {
            SelectionState::SignerLevel { signers, .. } => signers.clone(),
            SelectionState::AccountLevel { signers, .. } => signers.clone(),
            SelectionState::ChainLevel { signers, .. } => signers.clone(),
        };
        state.state = SelectionState::AccountLevel {
            signers: signers.clone(),
            signer_idx,
            selected: None,
        };

        let accounts = &signers[signer_idx].accounts;
        s.call_on_name("right_select", |view: &mut SelectView<String>| {
            view.clear();
            for account in accounts {
                view.add_item_str(&account.name);
            }
        });
    })
    .with(|view| {
        for (idx, signer) in signers.iter().enumerate() {
            view.add_item(signer.name.clone(), idx);
        }
    })
    .with_name("left_select")
    .scrollable()
    .fixed_width(20);
    // Create right pane SelectView for accounts (initially empty)
    let right_select = SelectView::<String>::new()
        .with_name("right_select")
        .scrollable()
        .fixed_width(20);
    
    // Define the vertical layouts for each pane
    let left_pane = LinearLayout::vertical()
        .child(TextView::new("Signers").center())
        .child(left_select);

    let right_pane = LinearLayout::vertical()
        .child(TextView::new("Accounts").center())
        .child(right_select);

    // Combine panes in a horizontal layout with panels
    let content = LinearLayout::horizontal()
        .child(Panel::new(left_pane).title("Signers"))
        .child(Panel::new(right_pane).title("Accounts"));

    // Add a single dialog containing both panes
    siv.add_layer(
        Dialog::new()
            .title("Signer and Account Selector")
            .content(content),
    );

    siv.run();}
