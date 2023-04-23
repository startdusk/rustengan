.PHONY: codeline
codeline:
	@tokei .

.PHONY: test 
test: fmt
	@cargo nextest run

.PHONY: fmt
fmt:
	@cargo fmt
	@cargo fmt -- --check
	@cargo clippy --all-targets --all-features --tests --benches -- -D warnings

.PHONY: echo
echo:
	@./maelstrom/maelstrom test -w echo --bin ./target/debug/echo --node-count 1 --time-limit 10 

.PHONY: unique-ids
unique-ids:
	@./maelstrom/maelstrom test -w unique-ids --bin ./target/debug/unique-ids --time-limit 30 --rate 1000 --node-count 3 --availability total --nemesis partition

.PHONY: broadcast
broadcast: 
# 单节点广播
	@./maelstrom/maelstrom test -w broadcast --bin ./target/debug/broadcast --node-count 1 --time-limit 20 --rate 10
# 多节点广播
	@./maelstrom/maelstrom test -w broadcast --bin ./target/debug/broadcast --node-count 5 --time-limit 20 --rate 10
# 多节点广播 - 网络分区
	@./maelstrom/maelstrom test -w broadcast --bin ./target/debug/broadcast --node-count 5 --time-limit 20 --rate 10 --nemesis partition

# for debugging maelstrom
.PHONY: serve
serve:
	@./maelstrom/maelstrom serve
