#!/usr/bin/env python3
# MLX Model Testing Script

import os
import sys
import time
import argparse
import json
from pathlib import Path

# Add the project root to Python path
project_root = Path(__file__).resolve().parent.parent
sys.path.append(str(project_root))

try:
    import mlx.core as mx
    import mlx.nn as nn
except ImportError:
    print("Error: MLX is not installed. Please install it with: pip install mlx>=0.24.2")
    sys.exit(1)

def parse_args():
    parser = argparse.ArgumentParser(description="Test MLX models for PostDevAI")
    parser.add_argument("--model-path", type=str, required=True, help="Path to MLX model")
    parser.add_argument("--tokenizer-path", type=str, required=True, help="Path to tokenizer")
    parser.add_argument("--model-type", type=str, default="auto", 
                        choices=["auto", "llama", "mistral", "phi", "qwen", "gemma"], 
                        help="Model type")
    parser.add_argument("--prompt", type=str, default="Explain the RAM-Lake architecture in PostDevAI", 
                        help="Test prompt")
    parser.add_argument("--max-tokens", type=int, default=512, help="Maximum tokens to generate")
    parser.add_argument("--device", type=str, default="gpu", choices=["cpu", "gpu"], 
                        help="Device to run on")
    parser.add_argument("--temperature", type=float, default=0.7, help="Sampling temperature")
    parser.add_argument("--top-p", type=float, default=0.9, help="Top-p sampling")
    return parser.parse_args()

def measure_memory_usage():
    """Measure current memory usage"""
    import psutil
    process = psutil.Process(os.getpid())
    memory_info = process.memory_info()
    return memory_info.rss / (1024 * 1024 * 1024)  # Convert to GB

def main():
    args = parse_args()
    
    print(f"Testing MLX model at: {args.model_path}")
    print(f"Using tokenizer at: {args.tokenizer_path}")
    print(f"Model type: {args.model_type}")
    print(f"Device: {args.device}")
    print()
    
    # Configure device
    if args.device == "gpu":
        mx.set_default_device(mx.gpu)
    else:
        mx.set_default_device(mx.cpu)
    
    # Load tokenizer
    print("Loading tokenizer...")
    try:
        from transformers import AutoTokenizer
        tokenizer = AutoTokenizer.from_pretrained(args.tokenizer_path)
    except ImportError:
        print("Error: transformers is not installed. Please install it with: pip install transformers")
        sys.exit(1)
    except Exception as e:
        print(f"Error loading tokenizer: {e}")
        sys.exit(1)
    
    # Load model
    print("Loading model...")
    memory_before = measure_memory_usage()
    start_time = time.time()
    
    try:
        model = nn.load_model(args.model_path)
    except Exception as e:
        print(f"Error loading model: {e}")
        sys.exit(1)
    
    load_time = time.time() - start_time
    memory_after = measure_memory_usage()
    memory_used = memory_after - memory_before
    
    print(f"Model loaded in {load_time:.2f} seconds")
    print(f"Memory usage: {memory_used:.2f} GB")
    print()
    
    # Basic model information
    params = sum(p.size for p in model.parameters())
    print(f"Model parameters: {params / 1e9:.2f}B")
    
    # Generate from prompt
    print("Generating from prompt:")
    print(f"Prompt: {args.prompt}")
    print()
    
    # Tokenize input
    input_tokens = mx.array(tokenizer.encode(args.prompt))
    
    # Set up generation config
    generation_args = {
        "max_tokens": args.max_tokens,
        "temperature": args.temperature,
        "top_p": args.top_p,
    }
    
    # Generate with MLX
    start_time = time.time()
    outputs = model.generate(input_tokens, **generation_args)
    generation_time = time.time() - start_time
    
    # Decode output
    generated_text = tokenizer.decode(outputs[0].tolist())
    
    # Calculate tokens per second
    tokens_generated = len(outputs[0])
    tokens_per_second = tokens_generated / generation_time
    
    print("Generated text:")
    print("=" * 50)
    print(generated_text)
    print("=" * 50)
    print()
    
    print(f"Generated {tokens_generated} tokens in {generation_time:.2f} seconds")
    print(f"Speed: {tokens_per_second:.2f} tokens/second")
    
    # Memory after generation
    memory_final = measure_memory_usage()
    memory_delta = memory_final - memory_after
    print(f"Additional memory after generation: {memory_delta:.2f} GB")
    print(f"Total memory usage: {memory_final:.2f} GB")
    
    # Create performance report
    performance = {
        "model_path": args.model_path,
        "model_type": args.model_type,
        "device": args.device,
        "parameters_billion": params / 1e9,
        "load_time_seconds": load_time,
        "memory_used_gb": memory_used,
        "tokens_generated": tokens_generated,
        "generation_time_seconds": generation_time,
        "tokens_per_second": tokens_per_second,
        "total_memory_usage_gb": memory_final,
    }
    
    # Save report
    report_path = Path(f"model_performance_{int(time.time())}.json")
    with open(report_path, "w") as f:
        json.dump(performance, f, indent=2)
    
    print(f"Performance report saved to: {report_path}")

if __name__ == "__main__":
    main()