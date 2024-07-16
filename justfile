release := "false"
release_flag := if release == "true" { "--release" } else { "" }

alias c := client
alias s := server
# List all the available commands
[private]
default:
    @just --list --unsorted

run: server

# Build the rust project
[group('rust')]
[macos]
client:
    cargo run {{ release_flag }} --bin client

# Run the rust project
[group('rust')]
[macos]
server:
    cargo run {{ release_flag }}