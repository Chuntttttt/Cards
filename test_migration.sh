#!/bin/bash

# Cards Project Migration Test Script
# Tests the migration from Poetry to uv

set -e  # Exit on any error

echo "🧪 Testing Cards project migration from Poetry to uv"
echo "=================================================="

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

function print_status() {
    echo -e "${GREEN}✓${NC} $1"
}

function print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

function print_error() {
    echo -e "${RED}✗${NC} $1"
}

# Cleanup function
cleanup() {
    if [ -d .venv ]; then
        print_warning "Cleaning up virtual environment"
        rm -rf .venv
    fi
}

# Set trap to cleanup on exit
trap cleanup EXIT

echo ""
echo "Step 1: Clean environment and create fresh venv"
echo "---------------------------------------------"

# Remove existing venv
if [ -d .venv ]; then
    rm -rf .venv
    print_status "Removed existing virtual environment"
fi

# Create new venv with Python 3.12
if command -v uv &> /dev/null; then
    uv venv --python 3.12
    print_status "Created new virtual environment with Python 3.12"
else
    print_error "uv not found. Please install uv first: curl -LsSf https://astral.sh/uv/install.sh | sh"
    exit 1
fi

echo ""
echo "Step 2: Install dependencies"
echo "--------------------------"

# Activate venv and install dependencies
source .venv/bin/activate

# Install main dependencies
uv sync --dev
print_status "Installed main dependencies with uv sync"

# Install additional dev dependencies
uv pip install pytest mypy ruff pyinstaller
print_status "Installed development dependencies"

# Verify Python version
PYTHON_VERSION=$(python --version)
print_status "Using $PYTHON_VERSION"

echo ""
echo "Step 3: Verify project configuration"
echo "-----------------------------------"

# Check that pyproject.toml is valid
python -c "import tomllib; f=open('pyproject.toml','rb'); tomllib.load(f); f.close()"
print_status "pyproject.toml is valid TOML"

# Check for uv-specific files and absence of Poetry files
if [ ! -f poetry.lock ]; then
    print_status "poetry.lock successfully removed"
else
    print_error "poetry.lock still exists"
    exit 1
fi

if [ -f uv.lock ]; then
    print_status "uv.lock exists"
elif [ -f .venv/pyvenv.cfg ]; then
    print_status "Virtual environment properly configured"
fi

echo ""
echo "Step 4: Test application functionality"
echo "------------------------------------"

# Test help command
python -m cards --help > /dev/null 2>&1
print_status "Application help command works"

# Test that we can import the package
python -c "import cards; print(f'Cards version: {cards.__version__}')"
print_status "Package import works correctly"

echo ""
echo "Step 5: Run tests"
echo "---------------"

# Run pytest
pytest tests/ -v
print_status "All tests pass"

echo ""
echo "Step 6: Code quality checks"
echo "-------------------------"

# Run ruff linting
ruff check .
print_status "Ruff linting passes"

# Run ruff formatting check
ruff format --check .
print_status "Code formatting is correct"

# Run mypy
mypy cards/
print_status "mypy type checking passes"

echo ""
echo "Step 7: Test PyInstaller build"
echo "-----------------------------"

# Clean any previous build
if [ -d build ]; then
    rm -rf build
fi
if [ -d dist ]; then
    rm -rf dist
fi

# Build with PyInstaller
pyinstaller --onefile cards/__main__.py --name cards > /dev/null 2>&1
print_status "PyInstaller build completed"

# Test the executable
./dist/cards --help > /dev/null 2>&1
print_status "Executable works correctly"

# Clean build artifacts
rm -rf build dist *.spec
print_status "Build artifacts cleaned up"

echo ""
echo "Step 8: Verify PyMuPDF functionality"
echo "-----------------------------------"

# Test that PyMuPDF can be imported and basic functionality works
python -c "
import fitz
doc = fitz.open()
page = doc.new_page()
print('PyMuPDF basic functionality works')
doc.close()
"
print_status "PyMuPDF functionality verified"

echo ""
echo "Step 9: Test dependency resolution"
echo "---------------------------------"

# Check that all dependencies are properly resolved
python -c "
import sys
import cards
import fitz
import pytest
import mypy
import ruff
print('All dependencies properly installed')
"
print_status "All dependencies properly resolved"

echo ""
echo "Step 10: Validate project structure"
echo "----------------------------------"

# Check that all necessary files exist
REQUIRED_FILES=(
    "pyproject.toml"
    "cards/__init__.py"
    "cards/__main__.py"
    "cards/cards.py"
    "tests/test_cards.py"
    ".gitignore"
)

for file in "${REQUIRED_FILES[@]}"; do
    if [ -f "$file" ]; then
        print_status "$file exists"
    else
        print_error "$file is missing"
        exit 1
    fi
done

echo ""
echo "🎉 Migration Test Complete!"
echo "=========================="
echo ""
print_status "All migration tests passed successfully!"
echo ""
echo "Summary of what was tested:"
echo "- ✓ Virtual environment creation with Python 3.12"
echo "- ✓ Dependency installation with uv"
echo "- ✓ Application functionality (help command, import)"
echo "- ✓ Test suite execution"
echo "- ✓ Code quality checks (ruff, mypy)"
echo "- ✓ PyInstaller executable build"
echo "- ✓ PyMuPDF functionality"
echo "- ✓ All required files present"
echo ""
echo "The Cards project has been successfully migrated from Poetry to uv!"
echo ""
echo "Key commands for development:"
echo "- Create venv: uv venv --python 3.12"
echo "- Install deps: uv sync --dev && uv pip install pytest mypy ruff pyinstaller"
echo "- Run tests: pytest tests/"
echo "- Lint code: ruff check ."
echo "- Format code: ruff format ."
echo "- Type check: mypy cards/"
echo "- Run app: python -m cards --help"
echo "- Build executable: pyinstaller --onefile cards/__main__.py --name cards"