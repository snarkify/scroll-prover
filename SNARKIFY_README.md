## How-to run Scroll sdk prover locally
1. `make snarkify`
2. download required parameters and assets into `params` and `assets` directory
3. set up envrionment variable
```shell
export SCROLL_PROVER_ASSETS_DIR=assets
```
4. run `./target/release/snarkify`
5. In a different shell, run `run_real_chunk.sh` to submit a sample job to the prover to generate proof

## How-to run Scroll sdk prover docker
1. `make snarkify`
2. `docker build -t scroll-sdk-prover:latest .`
3. `docker run -v /home/ubuntu/scroll/volume:/snarkify-data  -p 8080:8080 scroll-sdk-prover`
4. In a different shell, run `run_real_chunk.sh` to submit a sample job to the prover to generate proof

## How-to run deploy and run elastic Scroll sdk prover
ssh to gpu-5
1. `cd scroll/scroll-prover`
2. build the docker image following above instructions
3. `snarkify deploy --tag "{your_tag}" --image scroll-sdk-prover:latest`
4. Follow the printed instruction to check if the deployment is done, it should show {your_tag} if it is successful
5. `snarkify task create --file chunk_req.json` to create a new proof task
6. you should get a {task_id} if the task is created, then use `snarkify task log {task_id}` to stream the logs