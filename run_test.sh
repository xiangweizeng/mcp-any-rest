#!/bin/bash
# Run all Test bin
cargo run --bin test_user
cargo run --bin test_program
cargo run --bin test_product
cargo run --bin test_plan
cargo run --bin test_release # faild
cargo run --bin test_story
cargo run --bin test_project
cargo run --bin test_build
cargo run --bin test_task
cargo run --bin test_bug
cargo run --bin test_testcase # faild
cargo run --bin test_testtask # faild
