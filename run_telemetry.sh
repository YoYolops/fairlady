#!/bin/bash

# Exit the shell script immediately if any command fails midway
set -e

echo "============================================="
echo "  Fairlady Telemetry Automation Launcher"
echo "============================================="

# Verify Python 3 presence
if ! command -v python3 &> /dev/null; then
    echo "ERROR: Python 3 is not installed or not found in your PATH." >&2
    echo "Please install python3 to run the telemetry plotting modules." >&2
    exit 1
fi

echo "Python 3 detected."

# Define the virtual environment directory name
VENV_DIR=".venv"

# Check, create, and configure the Virtual Environment
if [ ! -d "$VENV_DIR" ]; then
    echo "Creating venv in '$VENV_DIR'..."
    python3 -m venv "$VENV_DIR"
fi

# Activate the virtual environment context
echo "Activating venv..."
source "$VENV_DIR/bin/activate"

# Verify and install dependencies from requirements.txt
if [ -f "requirements.txt" ]; then
    echo "Upgrading pip and installing venv dependencies..."
    pip install --quiet --upgrade pip
    pip install --quiet -r requirements.txt
    echo "Done."
else
    echo "WARNING: 'requirements.txt' was not found in this directory."
    echo "If the correct needed libraries are missing, the execution step below might crash."
fi

echo "============================================="

# Execute the telemetry script
if [ -f "telemetry.py" ]; then
    echo "Executing telemetry.py..."
    echo ""
    python3 telemetry.py
else
    echo "ERROR: Could not find 'telemetry.py' in the current directory." >&2
    exit 1
fi