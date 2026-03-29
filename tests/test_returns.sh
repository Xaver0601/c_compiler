#!/bin/bash
# Helper script to find the expected return value of a given test file/set

# Loop through all .c files in the specified directory
for file in tests/valid/ifelse/*.c; do
    # Check if file exists
    if [ -f "$file" ]; then
        # Compile the current file
        gcc "$file" -o temp_exec
        
        # Check if compilation succeeded
        if [ $? -eq 0 ]; then
            # Run the compiled program
            ./temp_exec
            
            # Capture the return value of the program
            ret_val=$?
            
            # Print the file name and its return value
            echo "File: $file | Return value: $ret_val"
            
            # Clean up the compiled executable
            rm temp_exec
        else
            echo "File: $file | Compilation failed!"
        fi
    fi
done