cargo zigbuild --release --target aarch64-unknown-linux-gnu
mkdir build
cp ./target/aarch64-unknown-linux-gnu/release/commit-checker ./build/bootstrap
sam deploy
rm -rf build
