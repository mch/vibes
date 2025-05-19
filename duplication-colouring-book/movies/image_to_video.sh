#!/bin/bash

# Check if FFmpeg is installed
if ! command -v ffmpeg &> /dev/null; then
    echo "FFmpeg is not installed. Installing via Homebrew..."
    if ! command -v brew &> /dev/null; then
        echo "Homebrew is not installed. Please install Homebrew first: https://brew.sh/"
        exit 1
    fi
    brew install ffmpeg
fi

# Function to show usage information
show_usage() {
    echo "Usage: $0 -i <input_directory> -o <output_file> [-f <format>] [-r <framerate>]"
    echo ""
    echo "Options:"
    echo "  -i <input_directory>  Directory containing images (jpg, png, etc.)"
    echo "  -o <output_file>      Output video file (e.g., output.mp4)"
    echo "  -f <format>           Image format extension (default: jpg)"
    echo "  -r <framerate>        Output video framerate (default: 24)"
    exit 1
}

# Parse command line arguments
INPUT_DIR=""
OUTPUT_FILE=""
FORMAT="jpg"
FRAMERATE=24

while getopts "i:o:f:r:" opt; do
    case $opt in
        i) INPUT_DIR="$OPTARG" ;;
        o) OUTPUT_FILE="$OPTARG" ;;
        f) FORMAT="$OPTARG" ;;
        r) FRAMERATE="$OPTARG" ;;
        *) show_usage ;;
    esac
done

# Check for required arguments
if [ -z "$INPUT_DIR" ] || [ -z "$OUTPUT_FILE" ]; then
    show_usage
fi

# Check if input directory exists
if [ ! -d "$INPUT_DIR" ]; then
    echo "Error: Input directory '$INPUT_DIR' not found."
    exit 1
fi

# Find all images with the specified format
images=($(find "$INPUT_DIR" -type f -name "*.$FORMAT" | awk -F 'Slide|.jpg' '{print $0 "\t" $2}' | sort -k2 -n | cut -f1))
total_images=${#images[@]}

if [ $total_images -eq 0 ]; then
    echo "No .$FORMAT images found in '$INPUT_DIR'"
    exit 1
fi

echo "Found $total_images images in '$INPUT_DIR'"

# Create temporary directory for FFmpeg input file
INPUT_FILE="input.txt"
truncate -s 0 $INPUT_FILE

# Calculate duration for each image based on position
# Even images: 4.0s -> 0.5s
# Odd images: 1.5s -> 0.25s
echo "Calculating dynamic durations for images..."

for (( i=0; i<$total_images; i++ )); do
    # Calculate progress (0 to 1)
    progress=$(echo "scale=6; $i / ($total_images - 1)" | bc -z)

    # If total_images is 1, set progress to 0
    if [ $total_images -eq 1 ]; then
        progress=0
    fi

    # Calculate duration based on even/odd position
    if (( i % 2 == 0 )); then
        # Even-numbered images (0-indexed): 4.0s -> 0.5s
        EVEN_FRAMES_MAX_TIME=4.0
        EVEN_FRAMES_MIN_TIME=0.5
        duration=$(echo "scale=6; 4.0 - (4.0 - 0.5) * $progress" | bc -z)
    else
        # Odd-numbered images: 1.5s -> 0.25s
        ODD_FRAMES_MAX_TIME=1.0
        ODD_FRAMES_MIN_TIME=0.25
        duration=$(echo "scale=6; 1.5 - (1.5 - 0.25) * $progress" | bc -z)
    fi

    # Write to input file with path and duration
    echo "file '${images[$i]}'" >> "$INPUT_FILE"
    echo "duration $duration" >> "$INPUT_FILE"

    # Print progress
    echo "Image $(($i + 1))/${total_images}: ${images[$i]} - Duration: $duration seconds"
done

# Add the last image without duration (required by FFmpeg)
echo "file '${images[$((total_images-1))]}'" >> "$INPUT_FILE"

# Create the video using FFmpeg
echo "Creating video..."
set -x
ffmpeg -f concat -safe 0 -i "$INPUT_FILE" -pix_fmt yuv420p "$OUTPUT_FILE"

echo "Video creation complete: $OUTPUT_FILE"
