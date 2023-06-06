FROM paritytech/contracts-ci-linux:c531ca3a-20230602

# This version of the contracts node contains chain extensions need in our integration tests
RUN cargo install --git https://github.com/hyperledger/solang-substrate-ci.git --branch substrate-integration --force --locked && \
    rm -rf "${CARGO_HOME}/registry" "${CARGO_HOME}/git"
