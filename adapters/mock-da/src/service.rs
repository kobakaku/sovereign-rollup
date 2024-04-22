use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock, RwLockWriteGuard};
use tokio::time;

use crate::types::MockHash;
use crate::types::{MockBlob, MockBlockHeader};
use crate::{
    types::MockBlock,
    verifier::{MockDaSpec, MockDaVerifier},
};
use async_trait::async_trait;
use rollup_interface::services::da::DaService;
use rollup_interface::services::da::SlotData;
use rollup_interface::state::da::BlockHeaderTrait;

const GENESIS_HEADER: MockBlockHeader = MockBlockHeader {
    prev_hash: MockHash([0; 32]),
    hash: MockHash([1; 32]),
    height: 0,
};

#[derive(Clone)]
pub struct MockDaService {
    sender_address: [u8; 32],
    blocks: Arc<RwLock<VecDeque<MockBlock>>>,
    /// Used for calculating correct finality from state of `blocks`
    finalized_header_sender: broadcast::Sender<MockBlockHeader>,
}

impl MockDaService {
    pub fn new(sender_address: [u8; 32]) -> Self {
        let (tx, rx1) = broadcast::channel(10);
        tokio::spawn(async move {
            let mut rx = rx1;
            while let Ok(header) = rx.recv().await {
                tracing::debug!("Finalized MockHeader: {:?}", header);
            }
        });
        Self {
            sender_address,
            blocks: Arc::new(RwLock::new(VecDeque::new())),
            finalized_header_sender: tx,
        }
    }

    fn add_blob(
        &self,
        blob: &[u8],
        blocks: &mut RwLockWriteGuard<'_, VecDeque<MockBlock>>,
    ) -> anyhow::Result<()> {
        let (prev_block_hash, height) = match blocks.iter().last().map(|b| b.header()) {
            Some(block_header) => (block_header.hash(), block_header.height() + 1),
            None => (GENESIS_HEADER.hash(), GENESIS_HEADER.height() + 1),
        };

        let header = MockBlockHeader {
            prev_hash: prev_block_hash,
            // TODO: blobデータから有効なハッシュを作成する。
            hash: MockHash([123; 32]),
            height,
        };
        let blob = MockBlob::new();
        let block = MockBlock {
            header,
            blobs: vec![blob],
        };
        blocks.push_back(block);

        let next_index_to_fialize = blocks.len() - 1;
        let next_finalized_header = blocks[next_index_to_fialize].header().clone();
        self.finalized_header_sender
            .send(next_finalized_header)
            .unwrap();

        Ok(())
    }

    async fn wait_for_height(&self, height: u64) -> anyhow::Result<()> {
        /// Wait 100s
        for _ in 0..100000 {
            let blocks = self.blocks.read().await;
            if blocks.iter().any(|b| b.header().height() == height) {
                return Ok(());
            }
            time::sleep(Duration::from_millis(1)).await;
        }
        anyhow::bail!("No block at height={height} has been sent in {:?}s", 100)
    }
}

#[async_trait]
impl DaService for MockDaService {
    type Spec = MockDaSpec;

    type Verifier = MockDaVerifier;

    type Block = MockBlock;

    /// Gets block at given height
    async fn get_block_at(&self, height: u64) -> anyhow::Result<Self::Block> {
        if height == 0 {
            anyhow::bail!("The lowest queryable block should be > 0.")
        }

        // Waits
        self.wait_for_height(height).await?;

        let blocks = self.blocks.read().await;

        let index = height - 1;

        Ok(blocks.get(index as usize).unwrap().clone())
    }

    async fn get_last_finalized_block_header(
        &self,
    ) -> anyhow::Result<<Self::Spec as rollup_interface::state::da::DaSpec>::BlockHeader> {
        let blocks = self.blocks.read().await;
        if blocks.len() < 1 {
            return Ok(GENESIS_HEADER);
        }

        let index = blocks.len() - 1;
        Ok(blocks[index].header().clone())
    }

    async fn send_transaction(&self, blob: &[u8]) -> anyhow::Result<()> {
        let mut blocks = self.blocks.write().await;
        self.add_blob(blob, &mut blocks)?;
        Ok(())
    }
}
