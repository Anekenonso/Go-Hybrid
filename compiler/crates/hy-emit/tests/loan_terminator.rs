use hy_emit::emit_go;
use hy_syntax::parse_file;

#[test]
fn slot_loan_store_back_emits_before_valued_return_issue_203() {
    let src = "\
fn update(m map[string]int, k string) int {
    slot := &m[k]
    slot = 42
    return slot
}
fn main() {
    m := map[string]int{\"k\": 0}
    _ = update(m, \"k\")
}
";
    let ast = parse_file("loan_return.hy", src).expect("parses");
    let go = emit_go(&ast).expect("emits");
    let store_pos = go.find("m[k] = slot").expect("has store-back");
    let ret_pos = go.find("return slot").expect("has return");
    assert!(
        store_pos < ret_pos,
        "expected store-back before return slot, got:\n{}",
        go
    );
    assert_eq!(
        go.matches("m[k] = slot").count(),
        1,
        "expected no trailing duplicate store-back after return, got:\n{}",
        go
    );
}

#[test]
fn slot_loan_store_back_emits_before_early_void_return_issue_203() {
    let src = "\
fn bump_if_present(m map[string]int, k string) {
    if true {
        slot := &m[k]
        slot = 42
        return
    }
}
fn main() {
    m := map[string]int{\"k\": 0}
    bump_if_present(m, \"k\")
}
";
    let ast = parse_file("loan_lost_write.hy", src).expect("parses");
    let go = emit_go(&ast).expect("emits");
    let store_pos = go.find("m[k] = slot").expect("has store-back");
    let ret_pos = go.find("\t\treturn").expect("has return");
    assert!(
        store_pos < ret_pos,
        "expected store-back before early return, got:\n{}",
        go
    );
    assert_eq!(
        go.matches("m[k] = slot").count(),
        1,
        "expected no unreachable trailing store-back after return, got:\n{}",
        go
    );
}

#[test]
fn slot_loan_store_back_emits_before_break_and_continue_issue_203() {
    let src = "\
fn main() {
    m := map[string]int{\"k\": 0}
    for i in 0..2 {
        slot := &m[\"k\"]
        slot = i + 10
        break
    }
}
";
    let ast = parse_file("loan_break.hy", src).expect("parses");
    let go = emit_go(&ast).expect("emits");
    let store_pos = go.find("m[\"k\"] = slot").expect("has store-back");
    let brk_pos = go.find("\t\tbreak").expect("has break");
    assert!(
        store_pos < brk_pos,
        "expected store-back before break, got:\n{}",
        go
    );
    assert_eq!(
        go.matches("m[\"k\"] = slot").count(),
        1,
        "expected no unreachable trailing store-back after break, got:\n{}",
        go
    );
}

#[test]
fn slot_loan_store_back_emits_before_task_spawn_when_subsequent_range_loop_shadows_loan_name_issue_547() {
    let src = "\
struct item {
    x int
}
fn worker(m map[string]item, done chan bool) {
    done <- true
}
fn main() {
    m := map[string]item{\"a\": {x: 0}}
    done := make(chan bool, 1)
    n := &m[\"a\"]
    n.x = 1
    task worker(move m, done)
    total := 0
    for n in 0..1000 {
        total += n
    }
    <-done
}
";
    let ast = parse_file("loan_shadow_range.hy", src).expect("parses");
    let go = emit_go(&ast).expect("emits");
    let mut_pos = go.find("n.x = 1").expect("has loan mutation");
    let flush_pos = go.find("m[\"a\"] = n").expect("has store-back");
    let spawn_pos = go.find("go worker(m, done)").expect("has task spawn");
    assert!(
        mut_pos < flush_pos && flush_pos < spawn_pos,
        "store-back m[\"a\"] = n must be flushed before task spawn go worker(m, done), got:\n{}",
        go
    );
    assert_eq!(
        go.matches("m[\"a\"] = n").count(),
        1,
        "expected exactly one store-back, got:\n{}",
        go
    );
}

#[test]
fn slot_loan_store_back_emits_before_task_spawn_when_subsequent_iterate_loop_shadows_loan_name_issue_547() {
    let src = "\
struct item {
    x int
}
fn worker(m map[string]item, done chan bool) {
    done <- true
}
fn main() {
    m := map[string]item{\"a\": {x: 0}}
    done := make(chan bool, 1)
    n := &m[\"a\"]
    n.x = 1
    task worker(move m, done)
    items := []int{1, 2, 3}
    total := 0
    for n in items {
        total += n
    }
    <-done
}
";
    let ast = parse_file("loan_shadow_iterate.hy", src).expect("parses");
    let go = emit_go(&ast).expect("emits");
    let mut_pos = go.find("n.x = 1").expect("has loan mutation");
    let flush_pos = go.find("m[\"a\"] = n").expect("has store-back");
    let spawn_pos = go.find("go worker(m, done)").expect("has task spawn");
    assert!(
        mut_pos < flush_pos && flush_pos < spawn_pos,
        "store-back m[\"a\"] = n must be flushed before task spawn go worker(m, done), got:\n{}",
        go
    );
}

#[test]
fn slot_loan_store_back_emits_before_task_spawn_when_subsequent_select_recv_shadows_loan_name_issue_547() {
    let src = "\
struct item {
    x int
}
fn worker(m map[string]item, done chan bool) {
    done <- true
}
fn main() {
    m := map[string]item{\"a\": {x: 0}}
    done := make(chan bool, 1)
    n := &m[\"a\"]
    n.x = 1
    task worker(move m, done)
    sig := make(chan int, 1)
    sig <- 42
    select {
    n := <-sig:
        _ = n
    default:
    }
    <-done
}
";
    let ast = parse_file("loan_shadow_select.hy", src).expect("parses");
    let go = emit_go(&ast).expect("emits");
    let mut_pos = go.find("n.x = 1").expect("has loan mutation");
    let flush_pos = go.find("m[\"a\"] = n").expect("has store-back");
    let spawn_pos = go.find("go worker(m, done)").expect("has task spawn");
    assert!(
        mut_pos < flush_pos && flush_pos < spawn_pos,
        "store-back m[\"a\"] = n must be flushed before task spawn go worker(m, done), got:\n{}",
        go
    );
}
