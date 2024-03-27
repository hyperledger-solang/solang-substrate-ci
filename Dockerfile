FROM paritytech/ci-unified:bullseye-1.75.0

# This version of the contracts node contains chain extensions need in our integration tests
RUN cargo install --git https://github.com/hyperledger/solang-substrate-ci.git --branch substrate-integration --force --locked && \
    rm -rf "${CARGO_HOME}/registry" "${CARGO_HOME}/git"
