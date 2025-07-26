# OpenSubdiv-petite build commands

# Default build
build:
    cargo build

# Build with release optimizations
build-release:
    cargo build --release

# Build for Linux with clang-17 (recommended for Ubuntu/Debian)
build-linux-clang17:
    CC=clang-17 CXX=clang++-17 CXXFLAGS="-stdlib=libc++" RUSTFLAGS="-C link-arg=-stdlib=libc++ -C link-arg=-lc++abi" cargo build

# Build for Linux with clang-17 in release mode
build-linux-clang17-release:
    CC=clang-17 CXX=clang++-17 CXXFLAGS="-stdlib=libc++" RUSTFLAGS="-C link-arg=-stdlib=libc++ -C link-arg=-lc++abi" cargo build --release

# Run tests
test:
    cargo test

# Run tests with clang-17 (recommended for Ubuntu/Debian)
test-linux-clang17:
    CC=clang-17 CXX=clang++-17 CXXFLAGS="-stdlib=libc++" RUSTFLAGS="-C link-arg=-stdlib=libc++ -C link-arg=-lc++abi" cargo test

# Run tests with clang-17 and --nocapture for detailed output
test-linux-clang17-nocapture:
    CC=clang-17 CXX=clang++-17 CXXFLAGS="-stdlib=libc++" RUSTFLAGS="-C link-arg=-stdlib=libc++ -C link-arg=-lc++abi" cargo test -- --nocapture

# Run tests with clang-17 without default features (faster, no topology validation)
test-linux-clang17-no-default:
    CC=clang-17 CXX=clang++-17 CXXFLAGS="-stdlib=libc++" RUSTFLAGS="-C link-arg=-stdlib=libc++ -C link-arg=-lc++abi" cargo test --no-default-features

# Run specific test with clang-17
test-linux-clang17-specific test_name:
    CC=clang-17 CXX=clang++-17 CXXFLAGS="-stdlib=libc++" RUSTFLAGS="-C link-arg=-stdlib=libc++ -C link-arg=-lc++abi" cargo test {{test_name}}

# Run specific test with clang-17 and --nocapture
test-linux-clang17-specific-nocapture test_name:
    CC=clang-17 CXX=clang++-17 CXXFLAGS="-stdlib=libc++" RUSTFLAGS="-C link-arg=-stdlib=libc++ -C link-arg=-lc++abi" cargo test {{test_name}} -- --nocapture

# Check code without building
check:
    cargo check

# Clean build artifacts (keeps dependencies)
clean:
    cargo clean -p opensubdiv-petite -p opensubdiv-petite-sys

# Deep clean (removes entire target directory)
clean-all:
    cargo clean

# Build documentation
doc:
    cargo doc -p opensubdiv-petite --no-deps --open

# Build documentation without opening
doc-no-open:
    cargo doc -p opensubdiv-petite --no-deps

# Format code
fmt:
    cargo fmt

# Run clippy linter
clippy:
    cargo clippy --fix --allow-dirty

# Run a specific example
run-example example:
    cargo run --example {{example}}

# Run a specific example with clang-17
run-example-linux-clang17 example:
    CC=clang-17 CXX=clang++-17 CXXFLAGS="-stdlib=libc++" RUSTFLAGS="-C link-arg=-stdlib=libc++ -C link-arg=-lc++abi" cargo run --example {{example}}

# Run CUDA example (requires CUDA feature)
run-cuda-example:
    cargo run --example osd_tutorial_0_cuda --features cuda

# Build with CUDA support
build-cuda:
    cargo build --features cuda

# Build with all features
build-all-features:
    cargo build --all-features