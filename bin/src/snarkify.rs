use async_trait::async_trait;
use itertools::Itertools;
use prover::{
    aggregator::Prover as BatchProver, config::AGG_DEGREES, zkevm::Prover as ChunkProver,
    BatchProof, BatchProvingTask, BlockTrace, ChunkProof, ChunkProvingTask,
};
use serde::{Deserialize, Serialize};
use snarkify_sdk::prover::ProofHandler;
use std::{
    io,
    string::ToString,
    sync::{Mutex, OnceLock},
};

const PARAMS_DIR: &str = "./params";
const ASSETS_DIR: &str = "./assets";

static CHUNK_PROVER: OnceLock<Mutex<ChunkProver<'static>>> = OnceLock::new();
static BATCH_PROVER: OnceLock<Mutex<BatchProver<'static>>> = OnceLock::new();

fn get_chunk_prover() -> &'static Mutex<ChunkProver<'static>> {
    CHUNK_PROVER.get_or_init(|| {
        let params_map = Box::leak(Box::new(
            prover::common::Prover::load_params_map(
                PARAMS_DIR,
                &[
                    *prover::config::INNER_DEGREE,
                    *prover::config::LAYER1_DEGREE,
                    *prover::config::LAYER2_DEGREE,
                ],
            )
            .to_owned(),
        ));
        Mutex::new(ChunkProver::from_params_and_assets(params_map, ASSETS_DIR))
    })
}

fn get_batch_prover() -> &'static Mutex<BatchProver<'static>> {
    BATCH_PROVER.get_or_init(|| {
        let params_map = Box::leak(Box::new(
            prover::common::Prover::load_params_map(
                PARAMS_DIR,
                &AGG_DEGREES.iter().copied().collect_vec(),
            )
            .to_owned(),
        ));
        Mutex::new(BatchProver::from_params_and_assets(params_map, ASSETS_DIR))
    })
}

#[derive(Debug, Eq, PartialEq, Deserialize)]
#[serde(from = "u8")]
pub enum CircuitType {
    CHUNK = 1,
    BATCH = 2,
    BUNDLE = 3,
}

impl From<u8> for CircuitType {
    fn from(value: u8) -> Self {
        match value {
            1 => CircuitType::CHUNK,
            2 => CircuitType::BATCH,
            3 => CircuitType::BUNDLE,
            _ => panic!("Invalid circuit type: {}", value),
        }
    }
}

#[derive(Deserialize)]
pub struct ProveRequest {
    pub circuit_type: CircuitType,
    pub circuit_version: String,
    pub hard_fork_name: String,
    pub task_data: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum Proof {
    Chunk(ChunkProof),
    Batch(BatchProof),
}

struct MyProofHandler;

#[async_trait]
impl ProofHandler for MyProofHandler {
    type Input = ProveRequest;
    type Output = Proof;
    type Error = String;

    async fn prove(req: Self::Input) -> Result<Self::Output, Self::Error> {
        match req.circuit_type {
            CircuitType::CHUNK => {
                let task_data: Vec<BlockTrace> =
                    serde_json::from_str(req.task_data.as_str()).unwrap();
                let task = ChunkProvingTask::from(task_data);
                let mut prover = get_chunk_prover().lock().unwrap();
                let proof = prover
                    .gen_chunk_proof(task, None, None, None)
                    .map_err(|e| e.to_string())?;
                Ok(Proof::Chunk(proof))
            }
            CircuitType::BATCH => {
                let task_data: BatchProvingTask =
                    serde_json::from_str(req.task_data.as_str()).map_err(|e| e.to_string())?;
                let mut prover = get_batch_prover().lock().unwrap();
                let proof = prover
                    .gen_batch_proof(task_data, None, None)
                    .map_err(|e| e.to_string())?;
                Ok(Proof::Batch(proof))
            }
            CircuitType::BUNDLE => {
                panic!("Bundle proof generation not implemented yet");
            }
        }
    }
}

fn main() -> Result<(), io::Error> {
    snarkify_sdk::run::<MyProofHandler>()
}
