set SCRIPT_DIR (dirname (status --current-filename))
cd $SCRIPT_DIR

set bin $argv[1]

if test -z "$bin"
    echo "Usage: run_profile.fish <profile_bitboard|profile_uci>"
    exit 1
end

cargo run --features rand,hotpath,hotpath-alloc,hotpath-mcp --bin $bin --release
