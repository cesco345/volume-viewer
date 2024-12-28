#!/bin/bash

# Check if ImageMagick is installed
if ! command -v magick &> /dev/null; then
    echo "Error: ImageMagick is not installed"
    echo "Please install it using:"
    echo "  macOS:     brew install imagemagick"
    echo "  Ubuntu:    sudo apt-get install imagemagick"
    exit 1
fi

usage() {
    echo "Usage: $0 input_file [output_file]"
    echo "Converts any image to a volume-viewer compatible TIFF format"
    echo "If output_file is not specified, it will use input_file_converted.tiff"
    exit 1
}

# Check if input file is provided
if [ $# -eq 0 ]; then
    usage
fi

input_file="$1"
if [ ! -f "$input_file" ]; then
    echo "Error: Input file '$input_file' does not exist"
    exit 1
fi

# Set output filename
if [ $# -ge 2 ]; then
    output_file="$2"
else
    filename=$(basename -- "$input_file")
    name="${filename%.*}"
    output_file="${name}_converted.tiff"
fi

echo "Converting $input_file to $output_file..."

# Convert to single-channel grayscale TIFF with explicit pixel format
magick "$input_file" \
    -colorspace gray \
    -depth 8 \
    -define tiff:bits-per-sample=8 \
    -define tiff:photometric=minisblack \
    -define quantum:format=unsigned \
    -compress none \
    -endian msb \
    "$output_file"

# Double-check the conversion resulted in a single-channel image
channels=$(magick identify -format "%[channels]" "$output_file")
if [[ "$channels" != "1" && "$channels" != "gray" ]]; then
    echo "Warning: Conversion may not have produced a proper single-channel image"
    echo "Channels: $channels"
fi

# Check if conversion was successful
if [ $? -eq 0 ]; then
    echo "Conversion successful!"
    echo "Output saved as: $output_file"
    
    # Print file information
    echo -e "\nFile information:"
    magick identify -verbose "$output_file" | grep -E "Format|Colorspace|Depth|Type|Compression|Channel"
else
    echo "Error: Conversion failed"
    exit 1
fi