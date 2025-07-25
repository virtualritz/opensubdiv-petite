#!/bin/bash

echo "Fixing C++ build environment for Ubuntu..."
echo "This script should be run with sudo: sudo ./fix_build_env.sh"
echo ""

# Update package lists
echo "Updating package lists..."
apt update

# Install essential build tools and libraries
echo "Installing build-essential, cmake, and pkg-config..."
apt install -y build-essential cmake pkg-config

# Install GCC and G++ with their standard libraries
echo "Installing GCC 14 and G++ 14 with standard libraries..."
# Force reinstall to fix any broken installations
apt install --reinstall -y gcc-14 g++-14 libstdc++-14-dev

# Install Clang with libc++ (alternative to libstdc++)
echo "Installing Clang with libc++..."
apt install -y clang libc++-dev libc++abi-dev

# Install additional development headers that might be missing
echo "Installing kernel headers..."
apt install -y linux-headers-$(uname -r)

# Set up alternatives for gcc/g++ if needed
echo "Setting up gcc-14/g++-14 as default..."
update-alternatives --install /usr/bin/gcc gcc /usr/bin/gcc-14 100
update-alternatives --install /usr/bin/g++ g++ /usr/bin/g++-14 100

# For clang, install an older, more stable version
echo "Installing Clang 17 (stable version)..."
apt install -y clang-17 libc++-17-dev libc++abi-17-dev
update-alternatives --install /usr/bin/clang clang /usr/bin/clang-17 100
update-alternatives --install /usr/bin/clang++ clang++ /usr/bin/clang++-17 100

echo ""
echo "Build environment setup complete!"
echo ""
echo "Now you can build the project with:"
echo "  CC=gcc CXX=g++ cargo build"
echo ""
echo "Or if that doesn't work:"
echo "  CC=clang-17 CXX=clang++-17 cargo build"