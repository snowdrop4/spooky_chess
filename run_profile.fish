set SCRIPT_DIR (dirname (status --current-filename))
cd $SCRIPT_DIR

cargo run --features hotpath,hotpath-alloc,hotpath-mcp --bin profile
