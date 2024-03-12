P=$PWD

set -e

cd front
pnpm run build

cd $P
cd server

cargo run -- --config ../data/config.example.toml
