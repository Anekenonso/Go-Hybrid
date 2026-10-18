//! channel_direction_coercion.rs — regression for issue #623: channel direction
//! was never checked at value-flow coercion sites (ascriptions, assignments,
//! call/task/method arguments, return statements, struct-literal fields, map stores,
//! and channel-of-channel sends). Emitted Go failed `go vet` with
//! `cannot use ... as ... value`.
//!
//! Pins HYB-E3012.

use hy_check::{check_file, Verdict};
use hy_diag::Severity;
use hy_syntax::parse_file;

fn check(src: &str) -> hy_check::CheckResult {
    let path = "inline.hy";
    let ast = parse_file(path, src)
        .unwrap_or_else(|diags| panic!("must parse (parser contract): {:?}", diags));
    check_file(&ast)
}

fn error_codes(r: &hy_check::CheckResult) -> Vec<String> {
    r.diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .map(|d| d.code.clone())
        .collect()
}

#[test]
fn repro_1_var_decl_ascription_cross_unidir_rejects_e3012() {
    let src = "\
fn main() {
    var sc chan<- int = make(chan int, 1)
    var rc <-chan int = sc
    _ = rc
}
";
    let r = check(src);
    assert_eq!(r.verdict, Verdict::Reject);
    let cs = error_codes(&r);
    assert!(
        cs.iter().any(|c| c == "HYB-E3012"),
        "expected HYB-E3012 on cross-unidir ascription, got {:?} diags={:?}",
        cs,
        r.diagnostics
    );
}

#[test]
fn repro_2_var_decl_ascription_widening_rejects_e3012() {
    let src = "\
fn main() {
    var sc chan<- int = make(chan int, 1)
    var bc chan int = sc
    _ = bc
}
";
    let r = check(src);
    assert_eq!(r.verdict, Verdict::Reject);
    let cs = error_codes(&r);
    assert!(
        cs.iter().any(|c| c == "HYB-E3012"),
        "expected HYB-E3012 on unidir widening to bidir, got {:?} diags={:?}",
        cs,
        r.diagnostics
    );
}

#[test]
fn repro_3_assignment_cross_unidir_rejects_e3012() {
    let src = "\
fn main() {
    var sc chan<- int = make(chan int, 1)
    var rc <-chan int
    rc = sc
    _ = rc
}
";
    let r = check(src);
    assert_eq!(r.verdict, Verdict::Reject);
    let cs = error_codes(&r);
    assert!(
        cs.iter().any(|c| c == "HYB-E3012"),
        "expected HYB-E3012 on cross-unidir assignment, got {:?} diags={:?}",
        cs,
        r.diagnostics
    );
}

#[test]
fn repro_4_call_argument_rejects_e3012() {
    let src = "\
fn take_rc(rc <-chan int) {
    var v = <-rc
    _ = v
}
fn main() {
    var sc chan<- int = make(chan int, 1)
    take_rc(sc)
}
";
    let r = check(src);
    assert_eq!(r.verdict, Verdict::Reject);
    let cs = error_codes(&r);
    assert!(
        cs.iter().any(|c| c == "HYB-E3012"),
        "expected HYB-E3012 on call argument direction mismatch, got {:?} diags={:?}",
        cs,
        r.diagnostics
    );
}

#[test]
fn repro_5_return_statement_rejects_e3012() {
    let src = "\
fn get_rc() <-chan int {
    var sc chan<- int = make(chan int, 1)
    return sc
}
fn main() {
    var rc = get_rc()
    _ = rc
}
";
    let r = check(src);
    assert_eq!(r.verdict, Verdict::Reject);
    let cs = error_codes(&r);
    assert!(
        cs.iter().any(|c| c == "HYB-E3012"),
        "expected HYB-E3012 on return statement direction mismatch, got {:?} diags={:?}",
        cs,
        r.diagnostics
    );
}

#[test]
fn repro_6_struct_literal_field_rejects_e3012() {
    let src = "\
struct holder { ch <-chan int }
fn main() {
    var sc chan<- int = make(chan int, 1)
    h := holder{ch: sc}
    _ = h
}
";
    let r = check(src);
    assert_eq!(r.verdict, Verdict::Reject);
    let cs = error_codes(&r);
    assert!(
        cs.iter().any(|c| c == "HYB-E3012"),
        "expected HYB-E3012 on struct-literal field direction mismatch, got {:?} diags={:?}",
        cs,
        r.diagnostics
    );
}

#[test]
fn repro_7_map_store_rejects_e3012() {
    let src = "\
fn main() {
    var m = map[string]<-chan int{}
    var ch = make(chan<- int, 1)
    m[\"a\"] = ch
    var v = <-m[\"a\"]
    _ = v
}
";
    let r = check(src);
    assert_eq!(r.verdict, Verdict::Reject);
    let cs = error_codes(&r);
    assert!(
        cs.iter().any(|c| c == "HYB-E3012"),
        "expected HYB-E3012 on map store direction mismatch, got {:?} diags={:?}",
        cs,
        r.diagnostics
    );
}

#[test]
fn repro_8_task_spawn_argument_rejects_e3012() {
    let src = "\
fn work(rc <-chan int) {
    var v = <-rc
    _ = v
}
fn main() {
    var sc chan<- int = make(chan int, 1)
    task work(sc)
}
";
    let r = check(src);
    assert_eq!(r.verdict, Verdict::Reject);
    let cs = error_codes(&r);
    assert!(
        cs.iter().any(|c| c == "HYB-E3012"),
        "expected HYB-E3012 on task spawn argument direction mismatch, got {:?} diags={:?}",
        cs,
        r.diagnostics
    );
}

#[test]
fn repro_9_closure_arg_and_return_rejects_e3012() {
    let src = "\
fn main() {
    var sc chan<- int = make(chan int, 1)
    run := fn(ch <-chan int) {
        var v = <-ch
        _ = v
    }
    run(sc)
    get := fn() <-chan int {
        return sc
    }
    _ = get
}
";
    let r = check(src);
    assert_eq!(r.verdict, Verdict::Reject);
    let cs = error_codes(&r);
    assert!(
        cs.iter().any(|c| c == "HYB-E3012"),
        "expected HYB-E3012 on closure argument and return direction mismatch, got {:?} diags={:?}",
        cs,
        r.diagnostics
    );
}

#[test]
fn repro_10_method_argument_rejects_e3012() {
    let src = "\
struct box { ch <-chan int }
fn (b box) take(rc <-chan int) {
    var v = <-rc
    _ = v
}
fn main() {
    var sc chan<- int = make(chan int, 1)
    b := box{}
    b.take(sc)
    h := box{ch: sc}
    _ = h
}
";
    let r = check(src);
    assert_eq!(r.verdict, Verdict::Reject);
    let cs = error_codes(&r);
    assert!(
        cs.iter().any(|c| c == "HYB-E3012"),
        "expected HYB-E3012 on method argument direction mismatch, got {:?} diags={:?}",
        cs,
        r.diagnostics
    );
}

#[test]
fn repro_11_chan_of_chan_send_rejects_e3012() {
    let src = "\
fn main() {
    outer := make(chan chan<- int, 1)
    var rc <-chan int = make(chan int, 1)
    outer <- rc
    var got <-chan int
    got = <-outer
    _ = got
}
";
    let r = check(src);
    assert_eq!(r.verdict, Verdict::Reject);
    let cs = error_codes(&r);
    assert!(
        cs.iter().any(|c| c == "HYB-E3012"),
        "expected HYB-E3012 on chan-of-chan send direction mismatch, got {:?} diags={:?}",
        cs,
        r.diagnostics
    );
}

#[test]
fn controls_bidirectional_channel_widens_to_unidirectional_ok() {
    let src = "\
fn take_bidir(c chan int) {}
fn take_recv(rc <-chan int) {}
fn take_send(sc chan<- int) {}

fn main() {
    c := make(chan int, 1)
    var sc chan<- int = c
    var rc <-chan int = c
    var bc chan int = c
    take_bidir(c)
    take_recv(c)
    take_send(c)
    _ = sc
    _ = rc
    _ = bc
}
";
    let r = check(src);
    let cs = error_codes(&r);
    assert!(
        !cs.iter().any(|c| c == "HYB-E3012"),
        "legal bidirectional widenings must not emit HYB-E3012, got {:?}",
        cs
    );
}
