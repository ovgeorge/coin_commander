use cursive::view::{Nameable, Scrollable};
use cursive::{
    traits::*,
    views::{Button, Dialog, LinearLayout, Panel, StackView, TextView},
    Cursive,
};
use cursive_tree_view::{Placement, TreeView};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs;
use std::io::Write;

// Newtype wrapper for tree items
#[derive(Debug, Clone)]
struct TreeItem(String, Vec<usize>);

impl fmt::Display for TreeItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

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

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
enum ViewMode {
    SignerBased,
    ChainBased,
}

#[derive(Debug, Serialize, Deserialize)]
struct SystemState {
    signers: Vec<Signer>,
    selected_path: Option<Vec<usize>>,
    view_mode: ViewMode,
    chain_names: Vec<String>,
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

    let mut chain_names = HashSet::new();
    for signer in &wallet_data.signers {
        for account in &signer.accounts {
            for chain in &account.chains {
                chain_names.insert(chain.name.clone());
            }
        }
    }
    let mut chain_names: Vec<String> = chain_names.into_iter().collect();
    chain_names.sort();

    Ok(SystemState {
        signers: wallet_data.signers,
        selected_path: None,
        view_mode: ViewMode::SignerBased,
        chain_names,
    })
}

fn build_tree(signers: &[Signer]) -> TreeView<TreeItem> {
    let mut tree = TreeView::new();

    for (signer_idx, signer) in signers.iter().enumerate() {
        let signer_row = tree
            .insert_item(
                TreeItem(signer.name.clone(), vec![signer_idx]),
                Placement::After,
                0,
            )
            .expect("Failed to insert signer");

        for (account_idx, account) in signer.accounts.iter().enumerate() {
            let account_row = tree
                .insert_item(
                    TreeItem(account.name.clone(), vec![signer_idx, account_idx]),
                    Placement::LastChild,
                    signer_row,
                )
                .expect("Failed to insert account");

            for (chain_idx, chain) in account.chains.iter().enumerate() {
                let chain_row = tree
                    .insert_item(
                        TreeItem(chain.name.clone(), vec![signer_idx, account_idx, chain_idx]),
                        Placement::LastChild,
                        account_row,
                    )
                    .expect("Failed to insert chain");

                for (asset_idx, asset) in chain.assets.iter().enumerate() {
                    let display = format!("{}: {}", asset.name, asset.amount);
                    tree.insert_item(
                        TreeItem(display, vec![signer_idx, account_idx, chain_idx, asset_idx]),
                        Placement::LastChild,
                        chain_row,
                    )
                    .expect("Failed to insert asset");
                }
            }
        }
    }

    tree
}

fn build_chain_tree(state: &SystemState) -> TreeView<TreeItem> {
    let mut tree = TreeView::new();
    let chain_index_map: HashMap<&String, usize> = state
        .chain_names
        .iter()
        .enumerate()
        .map(|(i, name)| (name, i))
        .collect();
    let mut chain_map: HashMap<usize, usize> = HashMap::new();
    let mut chain_signer_map: HashMap<usize, HashMap<String, usize>> = HashMap::new();
    let mut chain_signer_account_map: HashMap<(usize, String), HashMap<String, usize>> =
        HashMap::new();

    for (signer_idx, signer) in state.signers.iter().enumerate() {
        for (account_idx, account) in signer.accounts.iter().enumerate() {
            for (chain_idx, chain) in account.chains.iter().enumerate() {
                let chain_name = &chain.name;
                let chain_index = *chain_index_map.get(chain_name).unwrap();
                let chain_row = *chain_map.entry(chain_index).or_insert_with(|| {
                    tree.insert_item(
                        TreeItem(chain_name.clone(), vec![chain_index]),
                        Placement::After,
                        0,
                    )
                    .unwrap()
                });

                let signer_map = chain_signer_map
                    .entry(chain_index)
                    .or_insert_with(HashMap::new);
                let signer_row = *signer_map.entry(signer.name.clone()).or_insert_with(|| {
                    tree.insert_item(
                        TreeItem(signer.name.clone(), vec![chain_index, signer_idx]),
                        Placement::LastChild,
                        chain_row,
                    )
                    .unwrap()
                });

                let account_key = (chain_index, signer.name.clone());
                let account_map = chain_signer_account_map
                    .entry(account_key)
                    .or_insert_with(HashMap::new);
                let account_row = *account_map
                    .entry(account.name.clone())
                    .or_insert_with(|| {
                        tree.insert_item(
                            TreeItem(
                                account.name.clone(),
                                vec![chain_index, signer_idx, account_idx],
                            ),
                            Placement::LastChild,
                            signer_row,
                        )
                        .unwrap()
                    });

                for (asset_idx, asset) in chain.assets.iter().enumerate() {
                    let display = format!("{}: {}", asset.name, asset.amount);
                    tree.insert_item(
                        TreeItem(
                            display,
                            vec![chain_index, signer_idx, account_idx, chain_idx, asset_idx],
                        ),
                        Placement::LastChild,
                        account_row,
                    )
                    .unwrap();
                }
            }
        }
    }

    tree
}

fn get_details_signer_based(state: &SystemState, row: usize) -> (String, Vec<usize>) {
    let mut details = String::new();
    let mut current_path = Vec::new();

    let temp_tree = build_tree(&state.signers);
    if let Some(item) = temp_tree.borrow_item(row) {
        current_path = item.1.clone();
        match current_path.len() {
            1 => details = format!("Signer: {}", item.0),
            2 => details = format!("Account: {}", item.0),
            3 => details = format!("Chain: {}", item.0),
            4 => {
                if let Some(signer) = state.signers.get(current_path[0]) {
                    if let Some(account) = signer.accounts.get(current_path[1]) {
                        if let Some(chain) = account.chains.get(current_path[2]) {
                            if let Some(asset) = chain.assets.get(current_path[3]) {
                                details = format!("Asset: {}\nAmount: {}", asset.name, asset.amount);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    (details, current_path)
}

fn get_details_chain_based(state: &SystemState, row: usize) -> (String, Vec<usize>) {
    let mut details = String::new();
    let mut current_path = Vec::new();

    let temp_tree = build_chain_tree(state);
    if let Some(item) = temp_tree.borrow_item(row) {
        current_path = item.1.clone();
        match current_path.len() {
            1 => {
                if let Some(chain_name) = state.chain_names.get(current_path[0]) {
                    details = format!("Chain: {}", chain_name);
                }
            }
            2 => {
                if let Some(chain_name) = state.chain_names.get(current_path[0]) {
                    if let Some(signer) = state.signers.get(current_path[1]) {
                        details = format!("Signer: {} on chain {}", signer.name, chain_name);
                    }
                }
            }
            3 => {
                if let Some(chain_name) = state.chain_names.get(current_path[0]) {
                    if let Some(signer) = state.signers.get(current_path[1]) {
                        if let Some(account) = signer.accounts.get(current_path[2]) {
                            details = format!(
                                "Account: {} of signer {} on chain {}",
                                account.name, signer.name, chain_name
                            );
                        }
                    }
                }
            }
            5 => {
                if let Some(signer) = state.signers.get(current_path[1]) {
                    if let Some(account) = signer.accounts.get(current_path[2]) {
                        if let Some(chain) = account.chains.get(current_path[3]) {
                            if let Some(asset) = chain.assets.get(current_path[4]) {
                                details = format!(
                                    "Asset: {}\nAmount: {}\nChain: {}\nAccount: {}\nSigner: {}",
                                    asset.name, asset.amount, chain.name, account.name, signer.name
                                );
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    (details, current_path)
}

fn update_details(s: &mut Cursive, row: usize) {
    let state = s.user_data::<SystemState>().unwrap();
    let (details, current_path) = match state.view_mode {
        ViewMode::SignerBased => get_details_signer_based(state, row),
        ViewMode::ChainBased => get_details_chain_based(state, row),
    };
    state.selected_path = Some(current_path);
    s.call_on_name("details", |view: &mut TextView| {
        view.set_content(details);
    });
}

fn switch_view(s: &mut Cursive) {
    // Step 1: Mutate the state and drop the borrow
    {
        let state = s.user_data::<SystemState>().unwrap();
        match state.view_mode {
            ViewMode::SignerBased => state.view_mode = ViewMode::ChainBased,
            ViewMode::ChainBased => state.view_mode = ViewMode::SignerBased,
        }
        state.selected_path = None;
    } // Borrow of state ends here

    // Step 2: Update the UI with no overlapping borrow
    match s.user_data::<SystemState>().unwrap().view_mode {
        ViewMode::SignerBased => {
            s.call_on_name("tree_stack", |stack: &mut StackView| {
                stack.move_to_front(cursive::views::LayerPosition::FromBack(0)); // Switch to signer view
            });
        }
        ViewMode::ChainBased => {
            s.call_on_name("tree_stack", |stack: &mut StackView| {
                stack.move_to_front(cursive::views::LayerPosition::FromBack(1)); // Switch to chain view
            });
        }
    }

    s.call_on_name("details", |view: &mut TextView| {
        view.set_content("Select an item to see details");
    });
}

fn main() {
    let mut siv = cursive::default();

    let state = load_system_state().expect("Failed to load system state");
    let signer_tree = build_tree(&state.signers)
        .on_select(update_details)
        .with_name("signer_tree")
        .scrollable()
        .fixed_width(50);

    let chain_tree = build_chain_tree(&state)
        .on_select(update_details)
        .with_name("chain_tree")
        .scrollable()
        .fixed_width(50);

    siv.set_user_data(state);

    let signer_tree_panel = Panel::new(signer_tree)
        .title("Wallet Hierarchy")
        .with_name("signer_tree_panel");
    let chain_tree_panel = Panel::new(chain_tree)
        .title("Chain Hierarchy")
        .with_name("chain_tree_panel");

    let mut stacked_tree = StackView::new();
    stacked_tree.add_layer(signer_tree_panel);
    stacked_tree.add_layer(chain_tree_panel);

    let details_view = TextView::new("Select an item to see details")
        .with_name("details")
        .scrollable()
        .fixed_width(30);

    let content = LinearLayout::horizontal()
        .child(stacked_tree.with_name("tree_stack"))
        .child(Panel::new(details_view).title("Details"));

    siv.add_layer(
        Dialog::new()
            .title("Wallet Explorer")
            .content(
                LinearLayout::vertical()
                    .child(content)
                    .child(Button::new("Switch View", switch_view)),
            ),
    );

    siv.call_on_name("signer_tree", |tree: &mut TreeView<TreeItem>| {
        for row in 0..tree.len() {
            tree.expand_item(row);
        }
    });
    siv.call_on_name("chain_tree", |tree: &mut TreeView<TreeItem>| {
        for row in 0..tree.len() {
            tree.expand_item(row);
        }
    });

    siv.run();
}
