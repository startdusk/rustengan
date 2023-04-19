.PHONY: codeline
codeline:
	@tokei .

.PHONY: test 
test: fmt
	@cargo nextest run

.PHONY: fmt
fmt:
	@cargo fmt && cargo fmt -- --check && cargo clippy --all-targets --all-features --tests --benches -- -D warnings

.PHONY: echo
echo:
	@./maelstrom/maelstrom test -w echo --bin target/debug/rustengan --node-count 1 --time-limit 10 
