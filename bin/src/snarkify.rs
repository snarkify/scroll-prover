use std::io;
use std::string::ToString;
use std::sync::{OnceLock, Mutex};
use serde::{Deserialize, Serialize};
use snarkify_sdk::prover::ProofHandler;
use async_trait::async_trait;
use prover::{BlockTrace, ChunkProvingTask, zkevm::Prover as ChunkProver, ChunkProof};

const PARAMS_DIR: &str = "./params";
const ASSETS_DIR: &str = "./assets";

static PROVER: OnceLock<Mutex<ChunkProver<'static>>> = OnceLock::new();

fn get_chunk_prover() -> &'static Mutex<ChunkProver<'static>> {
    PROVER.get_or_init(|| {
        let params_map = Box::leak(Box::new(
            prover::common::Prover::load_params_map(
                PARAMS_DIR,
                &[
                    *prover::config::INNER_DEGREE,
                    *prover::config::LAYER1_DEGREE,
                    *prover::config::LAYER2_DEGREE,
                ],
            ).to_owned()
        ));
        Mutex::new(ChunkProver::from_params_and_assets(params_map, ASSETS_DIR))
    })
}

struct MyProofHandler;

#[derive(Deserialize)]
pub struct ProveRequest {
    pub circuit_type: u8,
    pub circuit_version: String,
    pub hard_fork_name: String,
    pub task_data: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum Proof {
    Chunk(ChunkProof),
}

#[async_trait]
impl ProofHandler for MyProofHandler {
    type Input = ProveRequest;
    type Output = Proof;
    type Error = String;

    async fn prove(req: Self::Input) -> Result<Self::Output, Self::Error>
    {
        let task_data: Vec<BlockTrace> = serde_json::from_str(req.task_data.as_str()).unwrap();
        let task = ChunkProvingTask::from(task_data);
        let mut prover = get_chunk_prover().lock().unwrap();
        let proof = prover.gen_chunk_proof(task, None, None, None)
            .map_err(|e| e.to_string())?;
        Ok(Proof::Chunk(proof))
    }
}

fn main() -> Result<(), io::Error> {
    snarkify_sdk::run::<MyProofHandler>()
}
