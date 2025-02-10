#!/bin/bash

set -e

read -p "Do you want to build the project? (y/n): " choice
case "$choice" in 
    y|Y ) 
        echo -e "\nBuilding the project..."
        cargo build --release
        ;;
    n|N ) echo -e "\nSkipping the build.";;
    * ) 
        echo -e "\nInvalid input. Please enter y or n."
        exit 1
        ;;
esac

INSTALL_DIR="/usr/local/bin"

echo -e "\nCopying the binary to $INSTALL_DIR..."
sudo cp target/release/clai $INSTALL_DIR

echo -e "\nclai has been installed successfully!"

ALIASES_CREATED=$(grep -c "clai" ~/.bash_aliases)
if [ $ALIASES_CREATED -gt 0 ]; then
    exit 0
fi

echo ""
echo -e "\033[1mDo you want to add shortcuts to ~/.bash_aliases?\033[0m"
echo "chat    -> clai chat"
echo "suggest -> clai suggest"
echo "explain -> clai explain"
echo ""

read -p "Accept? (y/n): " choice
case "$choice" in 
    y|Y ) 
        echo "alias chat='clai chat'" >> ~/.bash_aliases
        echo "alias suggest='clai suggest'" >> ~/.bash_aliases
        echo "alias explain='clai explain'" >> ~/.bash_aliases
        ;;
esac