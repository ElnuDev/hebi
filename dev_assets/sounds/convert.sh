#!/bin/bash

# Convert all .wav files .mp3
# This must be done since Bevy doesn't support .wav files (yet)

# Get root of project from Git
# https://stackoverflow.com/a/957978
project_path=$(git rev-parse --show-toplevel)

# Get output sounds folder
sounds_folder=${project_path}/assets/sounds

# Get development sounds folder (where this script is located)
dev_sounds_folder=${project_path}/dev_assets/sounds

# Ensure the output sounds folder exists (-p)
mkdir -p ${sounds_folder}

# Clear out all previously generated .mp3 files in output sounds folder
rm ${sounds_folder}/assets/sounds/*.mp3

# Loop over all .wav files in development sounds folder
for i in ${dev_sounds_folder}/*.wav
	do
	# Get just the file name from the file path
	# https://stackoverflow.com/a/32372307
	file_name=$(echo $i | sed "s/.*\///")
	
	# Convert to .mp3 and save to output sounds folder
	# https://gist.github.com/championofblocks/3982727
	lame -b 320 -h "${i}" "${sounds_folder}/${file_name%.wav}.mp3"
done