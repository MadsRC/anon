# Anonymization SDK Makefile
# Automates builds and testing

.PHONY: help build test examples benchmark install check lint format clean all download-models download-gliner_small-v2.1 download-gliner-x-small compare-models

# Default target
help:
	@echo "🔒 Anonymization SDK - Available Commands"
	@echo ""
	@echo "🚀 Development:"
	@echo "  make build        - Build the project"
	@echo "  make test         - Run all tests"
	@echo "  make check        - Check code compilation"
	@echo "  make lint         - Run clippy linter"
	@echo "  make format       - Format code with rustfmt"
	@echo ""
	@echo "📚 Examples & Benchmarks:"
	@echo "  make examples     - Run all examples"
	@echo "  make demo-patterns - Run pattern-based demo"
	@echo "  make demo-gliner  - Run GLiNER demo (requires models)"
	@echo "  make benchmark    - Run performance benchmarks"
	@echo ""
	@echo "🤖 Model Management:"
	@echo "  make download-models           - Download all GLiNER models"
	@echo "  make download-gliner_small-v2.1 - Download GLiNER small v2.1 model"
	@echo "  make download-gliner-x-small   - Download GLiNER x-small model"
	@echo "  make compare-models            - Run comprehensive model comparison"
	@echo ""
	@echo "🧹 Cleanup:"
	@echo "  make clean        - Clean build artifacts"
	@echo ""
	@echo "⚡ Quick Start:"
	@echo "  make install      - Install dependencies + build"
	@echo "  make all          - Run full CI pipeline (build + test + lint)"


# Development Commands
build:
	@echo "🔨 Building project..."
	@cargo build --release
	@echo "✅ Build completed"

check:
	@echo "🔍 Checking code compilation..."
	@cargo check
	@echo "✅ Code checks passed"

test:
	@echo "🧪 Running tests..."
	@cargo test
	@echo "✅ Tests passed"

lint:
	@echo "🔍 Running clippy linter..."
	@cargo clippy -- -D warnings
	@echo "✅ Lint checks passed"

format:
	@echo "✨ Formatting code..."
	@cargo fmt
	@echo "✅ Code formatted"

# Examples
examples: demo-patterns demo-gliner

demo-patterns:
	@echo "🎯 Running pattern-based anonymization demo..."
	@cargo run --example patterns_demo

demo-gliner:
	@echo "🤖 Running GLiNER ML demo..."
	@cargo run --example gliner_demo

# Benchmarks
benchmark:
	@echo "⚡ Running performance benchmarks..."
	@cargo bench

# Installation & Setup
install: build
	@echo "🚀 Installation completed!"
	@echo ""
	@echo "📍 Binary location: target/release/anon"
	@echo ""
	@echo "🚀 Next steps:"
	@echo "  1. Download models: make download-models"
	@echo "  2. Setup models: mkdir -p $$HOME/.anon && cp -r models $$HOME/.anon/"
	@echo "  3. Test pattern demo: make demo-patterns"
	@echo "  4. Test GLiNER demo: make demo-gliner"
	@echo "  5. Test CLI: echo 'Hi John' | ./target/release/anon --model-path $$HOME/.anon/models/gliner/gliner-x-small"

# Cleanup
clean:
	@echo "🧹 Cleaning build artifacts..."
	@cargo clean
	@echo "✅ Build artifacts cleaned"


# CI Pipeline
all: check lint test build
	@echo "🎉 All checks passed! Ready for production."

# Model Downloads
download-models: models/gliner/gliner_small-v2.1/model.onnx models/gliner/gliner_small-v2.1/model_quantized.onnx models/gliner/gliner-x-small/model.onnx models/gliner/gliner-x-small/model_quantized.onnx
	@echo "🤖 All GLiNER models ready!"

models/gliner/gliner_small-v2.1/model.onnx:
	@echo "🤖 Downloading GLiNER small v2.1 model..."
	@mkdir -p models/gliner/gliner_small-v2.1
	@echo "https://huggingface.co/onnx-community/gliner_small-v2.1" > models/gliner/gliner_small-v2.1/source.txt
	@echo "📥 Downloading tokenizer..."
	@curl -L -o models/gliner/gliner_small-v2.1/tokenizer.json https://huggingface.co/onnx-community/gliner_small-v2.1/resolve/main/tokenizer.json
	@echo "📥 Downloading model..."
	@curl -L -o models/gliner/gliner_small-v2.1/model.onnx https://huggingface.co/onnx-community/gliner_small-v2.1/resolve/main/onnx/model.onnx
	@echo "✅ GLiNER small v2.1 model downloaded successfully!"

models/gliner/gliner_small-v2.1/model_quantized.onnx:
	@echo "🤖 Downloading GLiNER small v2.1 quantized model..."
	@mkdir -p models/gliner/gliner_small-v2.1
	@echo "📥 Downloading quantized model..."
	@curl -L -o models/gliner/gliner_small-v2.1/model_quantized.onnx https://huggingface.co/onnx-community/gliner_small-v2.1/resolve/main/onnx/model_quantized.onnx
	@echo "✅ GLiNER small v2.1 quantized model downloaded successfully!"

download-gliner_small-v2.1: models/gliner/gliner_small-v2.1/model.onnx models/gliner/gliner_small-v2.1/model_quantized.onnx

models/gliner/gliner-x-small/model.onnx:
	@echo "🤖 Downloading GLiNER x-small model..."
	@mkdir -p models/gliner/gliner-x-small
	@echo "https://huggingface.co/knowledgator/gliner-x-small" > models/gliner/gliner-x-small/source.txt
	@echo "📥 Downloading tokenizer..."
	@curl -L -o models/gliner/gliner-x-small/tokenizer.json https://huggingface.co/knowledgator/gliner-x-small/resolve/main/tokenizer.json
	@echo "📥 Downloading model..."
	@curl -L -o models/gliner/gliner-x-small/model.onnx https://huggingface.co/knowledgator/gliner-x-small/resolve/main/onnx/model.onnx
	@echo "✅ GLiNER x-small model downloaded successfully!"

models/gliner/gliner-x-small/model_quantized.onnx:
	@echo "🤖 Downloading GLiNER x-small quantized model..."
	@mkdir -p models/gliner/gliner-x-small
	@echo "📥 Downloading quantized model..."
	@curl -L -o models/gliner/gliner-x-small/model_quantized.onnx https://huggingface.co/knowledgator/gliner-x-small/resolve/main/onnx/model_quantized.onnx
	@echo "✅ GLiNER x-small quantized model downloaded successfully!"

download-gliner-x-small: models/gliner/gliner-x-small/model.onnx models/gliner/gliner-x-small/model_quantized.onnx

# Model Comparison and Evaluation
compare-models: download-models
	@cargo run --bin model_comparison


# Development workflow
dev:
	@echo "🛠️  Development setup complete"
	@echo "Ready for: make demo-patterns or make demo-gliner"
