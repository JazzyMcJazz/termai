#!/bin/bash

set -e

read -p "Do you want to build the project? (y/n): " choice
case "$choice" in 
    y|Y ) 
        echo "Building the project..."
        cargo build --release
        ;;
    n|N ) echo "Skipping the build.";;
    * ) 
        echo "Invalid input. Please enter y or n."
        exit 1
        ;;
esac

INSTALL_DIR="/usr/local/bin"

echo "Copying the binary to $INSTALL_DIR..."
sudo cp target/release/clai $INSTALL_DIR

echo "clai has been installed successfully!"

read -p "Create aliases? (y/n): " choice
case "$choice" in 
    y|Y ) 
        echo "alias chat='clai chat'" >> ~/.bash_aliases
        echo "alias suggest='clai suggest'" >> ~/.bash_aliases
        echo "alias explain='clai explain'" >> ~/.bash_aliases
        ;;
esac