use std::fmt::Debug;

use clap::Parser;
use sov_cli::{
    wallet_state::WalletState,
    workflows::{rpc::RpcWorkFlows, transaction::TransactionWorkFlows},
};
use sov_modules_core::DispatchCall;

use crate::RollupBlueprint;

#[derive(clap::Parser)]
struct Cli {
    #[clap(subcommand)]
    workflow: Workflows,
}

#[derive(clap::Subcommand)]
enum Workflows {
    #[clap(subcommand)]
    Transaction(TransactionWorkFlows),
    #[clap(subcommand)]
    Rpc(RpcWorkFlows),
}

const WALLET_STATE_DIR: &str = "wallet_state.json";

pub trait WalletBlueprint: RollupBlueprint
where
    <Self as RollupBlueprint>::NativeContext: serde::Serialize + serde::de::DeserializeOwned,
    <<Self as RollupBlueprint>::NativeRuntime as DispatchCall>::Decodable:
        serde::Serialize + serde::de::DeserializeOwned,
{
    fn run_wallet() -> anyhow::Result<()> {
        let wallet_state = WalletState::<
            <Self as RollupBlueprint>::NativeContext,
            <<Self as RollupBlueprint>::NativeRuntime as DispatchCall>::Decodable,
        >::load(WALLET_STATE_DIR)?;

        let cli = Cli::parse();

        match cli.workflow {
            Workflows::Transaction(inner) => inner.run(&wallet_state),
            Workflows::Rpc(_) => todo!(),
        }
    }
}
