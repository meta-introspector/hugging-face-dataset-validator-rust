#!/usr/bin/env python3
"""
Convert rust-analyzer JSON datasets to Parquet format with size limits for Git LFS.
Splits large files into chunks under 10MB each.
"""

import json
import pandas as pd
import os
import sys
from pathlib import Path
import math

def convert_json_to_parquet(json_file_path, output_dir, max_size_mb=9):
    """Convert JSON file to Parquet, splitting if necessary to stay under size limit."""
    
    print(f"Processing {json_file_path}...")
    
    # Read JSON data
    with open(json_file_path, 'r') as f:
        data = json.load(f)
    
    print(f"Loaded {len(data)} records")
    
    # Convert to DataFrame
    df = pd.DataFrame(data)
    
    # Create output directory
    os.makedirs(output_dir, exist_ok=True)
    
    # Estimate size per record (rough approximation)
    sample_size = min(100, len(data))
    sample_df = df.head(sample_size)
    temp_file = f"{output_dir}/temp_sample.parquet"
    sample_df.to_parquet(temp_file)
    sample_size_mb = os.path.getsize(temp_file) / (1024 * 1024)
    os.remove(temp_file)
    
    size_per_record_mb = sample_size_mb / sample_size
    records_per_chunk = int((max_size_mb * 0.9) / size_per_record_mb)  # 90% of limit for safety
    
    print(f"Estimated {size_per_record_mb:.4f} MB per record")
    print(f"Will use {records_per_chunk} records per chunk")
    
    if len(data) <= records_per_chunk:
        # Single file
        output_file = f"{output_dir}/data.parquet"
        df.to_parquet(output_file, index=False)
        size_mb = os.path.getsize(output_file) / (1024 * 1024)
        print(f"Created single file: {output_file} ({size_mb:.2f} MB)")
        return [output_file]
    else:
        # Multiple chunks
        num_chunks = math.ceil(len(data) / records_per_chunk)
        output_files = []
        
        for i in range(num_chunks):
            start_idx = i * records_per_chunk
            end_idx = min((i + 1) * records_per_chunk, len(data))
            
            chunk_df = df.iloc[start_idx:end_idx]
            output_file = f"{output_dir}/data-{i:05d}-of-{num_chunks:05d}.parquet"
            chunk_df.to_parquet(output_file, index=False)
            
            size_mb = os.path.getsize(output_file) / (1024 * 1024)
            print(f"Created chunk {i+1}/{num_chunks}: {output_file} ({size_mb:.2f} MB, {len(chunk_df)} records)")
            output_files.append(output_file)
        
        return output_files

def main():
    if len(sys.argv) != 3:
        print("Usage: python convert_to_parquet.py <input_analysis_dir> <output_repo_dir>")
        sys.exit(1)
    
    input_dir = Path(sys.argv[1])
    output_dir = Path(sys.argv[2])
    
    if not input_dir.exists():
        print(f"Input directory {input_dir} does not exist")
        sys.exit(1)
    
    # Create output directory structure
    output_dir.mkdir(exist_ok=True)
    
    # Process each phase
    phases = ['parsing-phase', 'name_resolution-phase', 'type_inference-phase']
    
    total_files = 0
    total_size_mb = 0
    
    for phase in phases:
        phase_input = input_dir / phase / 'data.json'
        if not phase_input.exists():
            print(f"Warning: {phase_input} not found, skipping...")
            continue
        
        phase_output = output_dir / phase
        files = convert_json_to_parquet(str(phase_input), str(phase_output))
        
        phase_size = sum(os.path.getsize(f) for f in files) / (1024 * 1024)
        total_files += len(files)
        total_size_mb += phase_size
        
        print(f"Phase {phase}: {len(files)} files, {phase_size:.2f} MB total")
        print()
    
    print(f"Conversion complete!")
    print(f"Total: {total_files} Parquet files, {total_size_mb:.2f} MB")
    print(f"All files should be under 10MB for Git LFS compatibility")

if __name__ == "__main__":
    main()
