#!/bin/bash

# Check if tiffinfo is installed
if ! command -v tiffinfo &> /dev/null; then
    echo "Error: libtiff tools are not installed"
    echo "Please install them using:"
    echo "  macOS:     brew install libtiff"
    echo "  Ubuntu:    sudo apt-get install libtiff-tools"
    exit 1
fi

# Text colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

check_tiff() {
    local file="$1"
    local warnings=false
    
    echo -e "\n${BLUE}Analyzing: $file${NC}"
    
    # Get TIFF information
    local tiff_info=$(tiffinfo "$file")
    
    # Extract key properties
    local bit_depth=$(echo "$tiff_info" | grep "Bits/Sample" | awk '{print $2}')
    local samples_per_pixel=$(echo "$tiff_info" | grep "Samples/Pixel" | awk '{print $2}')
    local compression=$(echo "$tiff_info" | grep "Compression Scheme" | awk '{print $4}')
    local photometric=$(echo "$tiff_info" | grep "Photometric Interpretation" | awk '{print $4}')
    local dimensions=$(echo "$tiff_info" | grep "Image Width" | awk '{print $3 "x" $7}')
    
    echo -e "\n${BLUE}Image Properties:${NC}"
    echo "Format:         TIFF"
    echo "Bit Depth:      $bit_depth-bit"
    echo "Samples/Pixel:  $samples_per_pixel"
    echo "Color Type:     $photometric"
    echo "Compression:    $compression"
    echo "Dimensions:     $dimensions"
    
    echo -e "\n${BLUE}Format Analysis:${NC}"
    
    # Check bit depth - just inform, don't fail
    if [ "$bit_depth" = "16" ]; then
        echo -e "${GREEN}• Bit depth is 16 (optimal)${NC}"
    else
        echo -e "${YELLOW}• Bit depth is $bit_depth (different from standard 16-bit, but might work)${NC}"
        warnings=true
    fi
    
    # Check samples per pixel
    if [ "$samples_per_pixel" = "1" ]; then
        echo -e "${GREEN}• Single channel (optimal)${NC}"
    else
        echo -e "${YELLOW}• Multi-channel: $samples_per_pixel channels${NC}"
        warnings=true
    fi
    
    # Check photometric interpretation
    if [ "$photometric" = "min-is-black" ] || [ "$photometric" = "min-is-white" ]; then
        echo -e "${GREEN}• Grayscale format (optimal)${NC}"
    else
        echo -e "${YELLOW}• Color format: $photometric${NC}"
        warnings=true
    fi
    
    # Check compression
    if [ "$compression" = "none" ]; then
        echo -e "${GREEN}• Uncompressed (optimal)${NC}"
    else
        echo -e "${YELLOW}• Compressed: $compression${NC}"
        warnings=true
    fi
    
    echo -e "\n${BLUE}Summary:${NC}"
    if [ "$warnings" = true ]; then
        echo -e "${YELLOW}This image has some non-standard properties but might still work.${NC}"
        echo "If you experience issues, you can use the conversion script to convert to standard format."
    else
        echo -e "${GREEN}This image has optimal properties for the volume viewer.${NC}"
    fi
}

# Show usage if no arguments provided
if [ $# -eq 0 ]; then
    echo "Usage: $0 image1 [image2 ...]"
    echo "Analyzes TIFF images and shows their properties"
    exit 1
fi

# Process all provided images
for file in "$@"; do
    if [ -f "$file" ]; then
        check_tiff "$file"
    else
        echo -e "${RED}Error: File '$file' does not exist${NC}"
    fi
done