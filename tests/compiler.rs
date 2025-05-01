use brainv::{compiler::Compiler, vm::{bench_run, Op}};

#[test]
fn wraps_long_arithmetic_runs() {
    for count in [255, 256, 257, 1024] {
        let increment = format!("{}.", "+".repeat(count));
        let decrement = format!("{}.", "-".repeat(count));

        assert_eq!(bench_run(&increment, vec![]), vec![count as u8]);
        assert_eq!(bench_run(&decrement, vec![]), vec![0u8.wrapping_sub(count as u8)]);
    }
}

#[test]
fn preserves_long_pointer_movements() {
    for count in [255, 256, 257, 1024] {
        let right = ">".repeat(count);
        let left = "<".repeat(count);
        let program = format!("{}+{}+.{}.", right, left, right);

        assert_eq!(bench_run(&program, vec![]), vec![1, 1]);
    }
}

#[test]
#[should_panic(expected = "Unmatched '['")]
fn rejects_unmatched_opening_bracket() {
    Compiler::new("[[]").compile();
}

#[test]
#[should_panic(expected = "Unmatched ']'")]
fn rejects_unmatched_closing_bracket() {
    Compiler::new("[]]").compile();
}

#[test]
fn accepts_largest_jump_index() {
    let program = format!("{}[]", ".".repeat(65533));
    let code = Compiler::new(&program).compile();

    assert!(matches!(code[65534], Op::JmpIfZ(u16::MAX)));
    assert!(matches!(code[65535], Op::JmpIfNZ(65534)));
}

#[test]
#[should_panic(expected = "Program too large for jump indices")]
fn rejects_oversized_opening_index() {
    let program = format!("{}[]", ".".repeat(65535));
    Compiler::new(&program).compile();
}

#[test]
#[should_panic(expected = "Program too large for jump indices")]
fn rejects_oversized_closing_index() {
    let program = format!("[{}]", ".".repeat(65535));
    Compiler::new(&program).compile();
}
