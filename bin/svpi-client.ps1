# Ensure script continues to run
$continue = $true

# Check if 'svpi check' is successful
try {
    svpi check
} catch {
    Write-Host "svpi check failed, exiting..."
    exit 1
}

# Display help information
svpi help

# Start command input loop
while ($continue) {
    $cmd = Read-Host "Enter command"

    # Split command into array of arguments
    $commandArgs = $cmd -split ' '

    switch ($commandArgs[0]) {
        "exit" {
            exit 0
        }
        "clear" {
            Clear-Host
            continue
        }
        default {
            try {
                # Execute command with arguments
                svpi @commandArgs
            } catch {
                Write-Host "Command failed: $_"
            }
        }
    }
}
