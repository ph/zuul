# SPDX-FileCopyrightText: 2025 Pier-Hugues Pellerin <ph@heykimo.com>
#
# SPDX-License-Identifier: MIT

default:
	just build

build:
	cargo build

test:
	cargo test

check:
	reuse lint
	cargo clippy --all --all-features --tests -- -D warnings
	cargo fmt --all --check