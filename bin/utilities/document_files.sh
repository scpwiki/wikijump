#!/usr/bin/env bash

# This is a script to help automate WDBUGS-141 (Regex documentation).

# This script:
  # 1. Finds all files containing a regex function (preg_*)
  # 2. Opens the first file that does not appear in documented_files.txt
    # The file is opened in $EDITOR
    # If $EDITOR is vi/vim/nvim, the file will be opened to the first
    # occurrence of preg_
  # 3. When the file is closed, it is added to documented_files.txt
    # If you did not finish working on the file, remember to remove it!
    # You can exit Vim without adding the file to the completed files list by
    # forcing it to exit with an error code: :cq

# Run this script multiple times to eventually cover all files.
# Run this script from the base directory.

# Find "preg_" OR the regex given as first arg
default_find="preg_"
find=${1:-$default_find}
echo "Finding: $find"

# Find all the files that match the regex
readarray -t matching_files < <(ag $find . -l --ignore lib/zf)
echo "Found ${#matching_files[@]} matching files"

# Get the file with the list of completed files
changed_files="$(dirname $0)/documented_files"
echo "Completed files: $changed_files"

for file in ${#matching_files}; do
  if ! grep -q $file $changed_files; then
    echo "Opening $file"
    if $EDITOR +"/$find" $file; then
      echo "Completed $file; adding to list"
      echo $file >> $changed_files
    else
      echo "Aborted $file; not adding to list"
    fi
    break
  fi
done

num_matched=${#matching_files[@]}
num_changed=$(wc -l $changed_files)

echo "There are $((num_matched-num_changed)) files remaining."
