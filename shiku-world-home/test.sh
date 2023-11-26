#!/bin/bash

# Run cargo test and check its exit status
if cargo test; then
    echo "Cargo test completed successfully."

    # Assuming the script is inside shiku-world-home directory
    # Copy files from shiku-world-home/home/bindings to the target directory
    cp -r ./home/bindings/* ../shiku-world-medium/client/communication/api/bindings/
    cp -r ./home/blueprints/* ../shiku-world-medium/client/communication/api/blueprints/
    cp -r ./home/blueprints/* ../shiku-world-medium/ui/src/blueprints
    echo "Files copied successfully."
else
    echo "Cargo test failed, not copying files."
fi
