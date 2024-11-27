watch-test testname="":
	RUST_BACKTRACE=1 cargo watch -- cargo test --workspace  -- --nocapture -- {{testname}}
	
