//! foreign.rs — the builtin foreign-type and method-mode tables.
//!
//! ── TECH DEBT (post-MVP, tracked for the lead) ────────────────────────────
//! These builtin descriptor STUBS stand in for the real boundary descriptors
//! (interfaces/abi-types.md, OQ8): the mode of every foreign call is
//! hardcoded here as data. When the bridge lands, this module is replaced by
//! generated descriptor tables (`own` / `borrow` / `mutate` / `consume`
//! flags per API, exactly as `finish` is flagged `consume` today). Until
//! then: unknown receivers degrade to `CallMode::Unknown` = "no loan
//! implications" (permissive — never a false reject; LOAN rules only, this
//! is not a safety pass), and unknown foreign TYPES degrade to
//! `TypeShape::Unknown` — which since O4.9 (rule FC, 2026-10-13) is
//! FAIL-CLOSED at spawn/send boundaries: a foreign result the stubs don't
//! describe cannot silently cross (HYB-E3005). Describing more of the real
//! boundary is exactly what the generated descriptor tables will widen.
//! ──────────────────────────────────────────────────────────────────────────

use crate::model::TypeShape;

/// Foreign package roots recognized without an import line (the corpus uses
/// `strings`/`fmt` freely; imports add to this set at runtime).
pub(crate) const BUILTIN_PACKAGES: &[&str] = &[
    "serde_json",
    "flate2",
    "atomic",
    "strings",
    "fmt",
    "http",
    "httptest",
    "sync",
    "net",
    "io",
    "os",
    "time",
    "sort",
    "strconv",
    "errors",
    "bufio",
    "filepath",
    "path",
    "hex",
    "json",
    "sha3",
    "regex",
];

/// Curated Go packages on the MVP allowlist (SPEC M5).
pub(crate) const CURATED_GO_PACKAGES: &[&str] = &[
    "bufio",
    "encoding/hex",
    "encoding/json",
    "errors",
    "filepath",
    "fmt",
    "hex",
    "http",
    "httptest",
    "io",
    "json",
    "net",
    "net/http",
    "net/http/httptest",
    "os",
    "path",
    "path/filepath",
    "sort",
    "strconv",
    "strings",
    "sync",
    "sync/atomic",
    "atomic",
    "time",
];

/// Pinned Rust crates on the MVP allowlist (SPEC M6).
pub(crate) const CURATED_RUST_CRATES: &[&str] = &[
    "rust/flate2",
    "rust/regex",
    "rust/serde_json",
    "rust/sha3",
    "flate2",
    "regex",
    "serde_json",
    "sha3",
];

/// Is this import path within the curated allowlist?
pub(crate) fn is_curated_import(path: &str) -> bool {
    CURATED_GO_PACKAGES.contains(&path) || CURATED_RUST_CRATES.contains(&path)
}

/// Normalized root name of an import path.
pub(crate) fn import_root(path: &str) -> &str {
    match path {
        "net/http" => "http",
        "net/http/httptest" => "httptest",
        "sync/atomic" => "atomic",
        "path/filepath" => "filepath",
        "encoding/hex" => "hex",
        "encoding/json" => "json",
        "rust/serde_json" => "serde_json",
        "rust/flate2" => "flate2",
        "rust/sha3" => "sha3",
        "rust/regex" => "regex",
        other => other.rsplit('/').next().unwrap_or(other),
    }
}

/// Known exported members for curated foreign packages (M4/M5 resolution).
pub(crate) fn is_known_package_member(pkg: &str, member: &str) -> bool {
    let m = member.trim();
    match pkg {
        "fmt" => matches!(
            m,
            "print"
                | "printf"
                | "println"
                | "sprint"
                | "sprintf"
                | "sprintln"
                | "fprint"
                | "fprintf"
                | "fprintln"
                | "errorf"
                | "eprint"
                | "eprintf"
                | "eprintln"
                | "scan"
                | "scanf"
                | "scanln"
                | "sscan"
                | "sscanf"
                | "sscanln"
                | "fscan"
                | "fscanf"
                | "fscanln"
                | "stringer"
                | "formatter"
                | "gostringer"
                | "scanner"
                | "state"
        ),
        "io" => matches!(
            m,
            "eof"
                | "read_all"
                | "read_full"
                | "read_at_least"
                | "copy"
                | "copy_n"
                | "copy_buffer"
                | "write_string"
                | "limit_reader"
                | "multi_reader"
                | "multi_writer"
                | "tee_reader"
                | "new_section_reader"
                | "pipe"
                | "nop_closer"
                | "err_unexpected_eof"
                | "err_short_write"
                | "err_short_buffer"
                | "err_no_progress"
                | "err_closed_pipe"
                | "seek_start"
                | "seek_current"
                | "seek_end"
                | "reader"
                | "writer"
                | "closer"
                | "read_closer"
                | "write_closer"
                | "read_writer"
                | "read_write_closer"
                | "seeker"
                | "read_seeker"
                | "write_seeker"
                | "read_write_seeker"
                | "reader_at"
                | "writer_at"
                | "byte_reader"
                | "byte_scanner"
                | "byte_writer"
                | "rune_reader"
                | "rune_scanner"
                | "string_writer"
                | "section_reader"
                | "limited_reader"
                | "pipe_reader"
                | "pipe_writer"
        ),
        "os" => matches!(
            m,
            "args"
                | "stdout"
                | "stderr"
                | "stdin"
                | "open"
                | "create"
                | "open_file"
                | "read_file"
                | "write_file"
                | "read_dir"
                | "getwd"
                | "user_home_dir"
                | "user_cache_dir"
                | "user_config_dir"
                | "executable"
                | "lookup_env"
                | "getenv"
                | "setenv"
                | "unsetenv"
                | "clearenv"
                | "environ"
                | "stat"
                | "lstat"
                | "create_temp"
                | "mkdir_temp"
                | "mkdir"
                | "mkdir_all"
                | "remove"
                | "remove_all"
                | "rename"
                | "exit"
                | "temp_dir"
                | "hostname"
                | "is_exist"
                | "is_not_exist"
                | "is_permission"
                | "is_timeout"
                | "expand_env"
                | "chown"
                | "chmod"
                | "chdir"
                | "truncate"
                | "pipe"
                | "link"
                | "symlink"
                | "readlink"
                | "same_file"
                | "getpid"
                | "getppid"
                | "getuid"
                | "geteuid"
                | "getgid"
                | "getegid"
                | "getgroups"
                | "new_file"
                | "find_process"
                | "start_process"
                | "o_rdonly"
                | "o_wronly"
                | "o_rdwr"
                | "o_append"
                | "o_create"
                | "o_excl"
                | "o_sync"
                | "o_trunc"
                | "mode_dir"
                | "mode_append"
                | "mode_exclusive"
                | "mode_temporary"
                | "mode_symlink"
                | "mode_device"
                | "mode_named_pipe"
                | "mode_socket"
                | "mode_setuid"
                | "mode_setgid"
                | "mode_char_device"
                | "mode_sticky"
                | "mode_irregular"
                | "mode_type"
                | "mode_perm"
                | "dev_null"
                | "file"
                | "file_mode"
                | "file_info"
                | "dir_entry"
                | "path_error"
                | "process"
                | "process_state"
                | "syscall_error"
        ),
        "http" => matches!(
            m,
            "new_serve_mux"
                | "handler"
                | "handler_func"
                | "serve"
                | "serve_file"
                | "serve_content"
                | "set_cookie"
                | "get"
                | "post"
                | "post_form"
                | "head"
                | "listen_and_serve"
                | "listen_and_serve_tls"
                | "new_request"
                | "new_request_with_context"
                | "error"
                | "not_found"
                | "redirect"
                | "detect_content_type"
                | "parse_time"
                | "status_text"
                | "canonical_header_key"
                | "max_bytes_reader"
                | "parse_http_date"
                | "format_http_date"
                | "status_continue"
                | "status_switching_protocols"
                | "status_processing"
                | "status_early_hints"
                | "status_ok"
                | "status_created"
                | "status_accepted"
                | "status_non_authoritative_info"
                | "status_no_content"
                | "status_reset_content"
                | "status_partial_content"
                | "status_multi_status"
                | "status_already_reported"
                | "status_im_used"
                | "status_multiple_choices"
                | "status_moved_permanently"
                | "status_found"
                | "status_see_other"
                | "status_not_modified"
                | "status_use_proxy"
                | "status_temporary_redirect"
                | "status_permanent_redirect"
                | "status_bad_request"
                | "status_unauthorized"
                | "status_payment_required"
                | "status_forbidden"
                | "status_not_found"
                | "status_method_not_allowed"
                | "status_not_acceptable"
                | "status_proxy_auth_required"
                | "status_request_timeout"
                | "status_conflict"
                | "status_gone"
                | "status_length_required"
                | "status_precondition_failed"
                | "status_payload_too_large"
                | "status_uri_too_long"
                | "status_unsupported_media_type"
                | "status_range_not_satisfiable"
                | "status_expectation_failed"
                | "status_teapot"
                | "status_misdirected_request"
                | "status_unprocessable_entity"
                | "status_locked"
                | "status_failed_dependency"
                | "status_too_early"
                | "status_upgrade_required"
                | "status_precondition_required"
                | "status_too_many_requests"
                | "status_request_header_fields_too_large"
                | "status_unavailable_for_legal_reasons"
                | "status_internal_server_error"
                | "status_not_implemented"
                | "status_bad_gateway"
                | "status_service_unavailable"
                | "status_gateway_timeout"
                | "status_http_version_not_supported"
                | "status_variant_also_negotiates"
                | "status_insufficient_storage"
                | "status_loop_detected"
                | "status_not_extended"
                | "status_network_authentication_required"
                | "method_get"
                | "method_head"
                | "method_post"
                | "method_put"
                | "method_patch"
                | "method_delete"
                | "method_connect"
                | "method_options"
                | "method_trace"
                | "request"
                | "response_writer"
                | "response"
                | "client"
                | "server"
                | "serve_mux"
                | "header"
                | "cookie"
                | "transport"
                | "round_tripper"
        ),
        "httptest" => matches!(
            m,
            "get"
                | "post"
                | "new_recorder"
                | "new_server"
                | "new_tls_server"
                | "new_unstarted_server"
                | "new_request"
                | "new_request_with_context"
                | "response_recorder"
                | "server"
        ),
        "time" => matches!(
            m,
            "now"
                | "since"
                | "until"
                | "sleep"
                | "after"
                | "tick"
                | "new_ticker"
                | "new_timer"
                | "after_func"
                | "parse"
                | "parse_in_location"
                | "parse_duration"
                | "load_location"
                | "load_location_from_tz_data"
                | "fixed_zone"
                | "date"
                | "unix"
                | "unix_milli"
                | "unix_micro"
                | "unix_nano"
                | "milliseconds"
                | "seconds"
                | "minutes"
                | "hours"
                | "microseconds"
                | "nanoseconds"
                | "nanosecond"
                | "microsecond"
                | "millisecond"
                | "second"
                | "minute"
                | "hour"
                | "rfc3339"
                | "rfc3339_nano"
                | "rfc822"
                | "rfc822_z"
                | "rfc850"
                | "rfc1123"
                | "rfc1123_z"
                | "kitchen"
                | "stamp"
                | "stamp_milli"
                | "stamp_micro"
                | "stamp_nano"
                | "date_only"
                | "time_only"
                | "date_time"
                | "ansic"
                | "unix_date"
                | "ruby_date"
                | "duration"
                | "time"
                | "location"
                | "ticker"
                | "timer"
                | "month"
                | "weekday"
                | "january"
                | "february"
                | "march"
                | "april"
                | "may"
                | "june"
                | "july"
                | "august"
                | "september"
                | "october"
                | "november"
                | "december"
                | "sunday"
                | "monday"
                | "tuesday"
                | "wednesday"
                | "thursday"
                | "friday"
                | "saturday"
        ),
        "strings" => matches!(
            m,
            "split"
                | "split_n"
                | "split_after"
                | "split_after_n"
                | "fields"
                | "fields_func"
                | "join"
                | "has_prefix"
                | "has_suffix"
                | "contains"
                | "contains_any"
                | "contains_rune"
                | "contains_func"
                | "count"
                | "index"
                | "index_any"
                | "index_byte"
                | "index_rune"
                | "index_func"
                | "last_index"
                | "last_index_any"
                | "last_index_byte"
                | "last_index_func"
                | "cut"
                | "cut_prefix"
                | "cut_suffix"
                | "to_lower"
                | "to_lower_special"
                | "to_upper"
                | "to_upper_special"
                | "to_title"
                | "to_title_special"
                | "to_valid_utf8"
                | "title"
                | "trim"
                | "trim_left"
                | "trim_right"
                | "trim_space"
                | "trim_prefix"
                | "trim_suffix"
                | "trim_func"
                | "trim_left_func"
                | "trim_right_func"
                | "replace"
                | "replace_all"
                | "equal_fold"
                | "repeat"
                | "map"
                | "new_reader"
                | "new_replacer"
                | "clone"
                | "builder"
                | "reader"
                | "replacer"
        ),
        "strconv" => matches!(
            m,
            "atoi"
                | "itoa"
                | "parse_int"
                | "parse_uint"
                | "parse_float"
                | "parse_bool"
                | "parse_complex"
                | "format_int"
                | "format_uint"
                | "format_float"
                | "format_bool"
                | "format_complex"
                | "append_int"
                | "append_uint"
                | "append_float"
                | "append_bool"
                | "quote"
                | "quote_to_ascii"
                | "quote_rune"
                | "quote_rune_to_ascii"
                | "unquote"
                | "unquote_char"
                | "can_backquote"
                | "is_print"
                | "is_graphic"
                | "err_range"
                | "err_syntax"
                | "num_error"
        ),
        "sort" => matches!(
            m,
            "ints"
                | "float64s"
                | "strings"
                | "ints_are_sorted"
                | "float64s_are_sorted"
                | "strings_are_sorted"
                | "is_sorted"
                | "sort"
                | "stable"
                | "slice"
                | "slice_is_sorted"
                | "slice_stable"
                | "search"
                | "search_ints"
                | "search_float64s"
                | "search_strings"
                | "reverse"
                | "interface"
                | "int_slice"
                | "float64_slice"
                | "string_slice"
        ),
        "errors" => matches!(m, "new" | "is" | "as" | "unwrap" | "join"),
        "sync" => matches!(
            m,
            "mutex"
                | "rw_mutex"
                | "wait_group"
                | "once"
                | "once_func"
                | "once_value"
                | "once_values"
                | "pool"
                | "cond"
                | "map"
                | "new_cond"
                | "locker"
        ),
        "atomic" => matches!(
            m,
            "bool"
                | "int32"
                | "int64"
                | "uint32"
                | "uint64"
                | "uintptr"
                | "pointer"
                | "value"
                | "add_int32"
                | "add_int64"
                | "add_uint32"
                | "add_uint64"
                | "add_uintptr"
                | "compare_and_swap_int32"
                | "compare_and_swap_int64"
                | "compare_and_swap_uint32"
                | "compare_and_swap_uint64"
                | "compare_and_swap_uintptr"
                | "compare_and_swap_pointer"
                | "load_int32"
                | "load_int64"
                | "load_uint32"
                | "load_uint64"
                | "load_uintptr"
                | "load_pointer"
                | "store_int32"
                | "store_int64"
                | "store_uint32"
                | "store_uint64"
                | "store_uintptr"
                | "store_pointer"
                | "swap_int32"
                | "swap_int64"
                | "swap_uint32"
                | "swap_uint64"
                | "swap_uintptr"
                | "swap_pointer"
        ),
        "filepath" => matches!(
            m,
            "base"
                | "clean"
                | "dir"
                | "ext"
                | "is_abs"
                | "join"
                | "split"
                | "from_slash"
                | "to_slash"
                | "volume_name"
                | "match"
                | "glob"
                | "abs"
                | "rel"
                | "eval_symlinks"
                | "walk"
                | "walk_dir"
                | "separator"
                | "list_separator"
                | "err_bad_pattern"
                | "skip_dir"
                | "skip_all"
        ),
        "path" => matches!(
            m,
            "base"
                | "clean"
                | "dir"
                | "ext"
                | "is_abs"
                | "join"
                | "split"
                | "match"
                | "err_bad_pattern"
        ),
        "hex" => matches!(
            m,
            "encode"
                | "encode_to_string"
                | "decode"
                | "decode_string"
                | "encoded_len"
                | "decoded_len"
                | "dump"
                | "dumper"
                | "new_decoder"
                | "new_encoder"
                | "err_length"
        ),
        "json" => matches!(
            m,
            "marshal"
                | "marshal_indent"
                | "unmarshal"
                | "valid"
                | "new_encoder"
                | "new_decoder"
                | "raw_message"
                | "token"
                | "decoder"
                | "encoder"
                | "syntax_error"
                | "unmarshal_type_error"
        ),
        "bufio" => matches!(
            m,
            "new_reader"
                | "new_reader_size"
                | "new_writer"
                | "new_writer_size"
                | "new_read_writer"
                | "new_scanner"
                | "scan_bytes"
                | "scan_runes"
                | "scan_words"
                | "scan_lines"
                | "reader"
                | "writer"
                | "read_writer"
                | "scanner"
                | "err_invalid_unread_byte"
                | "err_invalid_unread_rune"
                | "err_buffer_full"
                | "err_negative_count"
                | "err_too_long"
                | "err_negative_advance"
                | "err_advance_too_far"
                | "err_bad_read_count"
        ),
        "net" => matches!(
            m,
            "listen"
                | "listen_tcp"
                | "listen_udp"
                | "listen_ip"
                | "listen_unix"
                | "listen_multicast_udp"
                | "dial"
                | "dial_timeout"
                | "dial_tcp"
                | "dial_udp"
                | "dial_ip"
                | "dial_unix"
                | "resolve_tcp_addr"
                | "resolve_udp_addr"
                | "resolve_ip_addr"
                | "resolve_unix_addr"
                | "lookup_host"
                | "lookup_ip"
                | "lookup_port"
                | "lookup_cname"
                | "lookup_srv"
                | "lookup_mx"
                | "lookup_txt"
                | "lookup_addr"
                | "split_host_port"
                | "join_host_port"
                | "parse_ip"
                | "parse_cidr"
                | "ipv4"
                | "ipv4_mask"
                | "ipv4_all_systems"
                | "ipv4_all_routers"
                | "ipv6_zero"
                | "ipv6_unspecified"
                | "ipv6_loopback"
                | "ipv6_interface_local_all_nodes"
                | "ipv6_link_local_all_nodes"
                | "ipv6_link_local_all_routers"
                | "ip"
                | "ip_mask"
                | "ip_net"
                | "tcp_addr"
                | "udp_addr"
                | "unix_addr"
                | "ip_addr"
                | "conn"
                | "listener"
                | "packet_conn"
                | "error"
                | "addr"
        ),
        "serde_json" => matches!(
            m,
            "parse"
                | "new_object"
                | "new_array"
                | "to_string_pretty"
                | "array"
                | "get"
                | "int"
                | "string"
                | "set"
                | "remove"
                | "value"
                | "view"
        ),
        "flate2" => matches!(m, "new_writer" | "write" | "finish" | "writer"),
        "sha3" => matches!(m, "digest256" | "stream_checksum"),
        "regex" => matches!(m, "is_match" | "is_valid" | "compile" | "match_string"),
        _ => false,
    }
}


/// How a call uses its receiver / arguments (O7 handle modes + O13 consume).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CallMode {
    /// Creates a fresh own handle (does not touch any existing binding).
    Produce,
    /// Borrow-mode call: creates a read loan on the receiver (A2 window).
    Borrow,
    /// Mutating-mode call: conflicts with live borrow loans (A2).
    Mutate,
    /// Consuming call: own-transfer INTO the API — receiver moved-from (A7).
    Consume,
    /// Go-side / no loan semantics (fmt, strings, atomics, ...).
    Safe,
    /// Not in the table: no loan implications (permissive).
    Unknown,
}

/// The SCALAR `sync/atomic` types — the only atomics that are class 2
/// (sendable-by-type). Each one holds a plain machine value behind
/// internally synchronized Load/Store/Add/Swap/CompareAndSwap, so sharing
/// it across tasks cannot alias mutable GC state.
pub(crate) const SCALAR_ATOMICS: &[&str] = &[
    "atomic.bool",
    "atomic.int32",
    "atomic.int64",
    "atomic.uint32",
    "atomic.uint64",
    "atomic.uintptr",
];

/// Is `name` a scalar atomic (class 2)?
pub(crate) fn is_scalar_atomic(name: &str) -> bool {
    SCALAR_ATOMICS.contains(&name)
}

/// Is `name` a `sync/atomic` type OUTSIDE the scalar set — `atomic.pointer[T]`,
/// `atomic.value`, or anything else under `atomic.`? GUARANTEES.md row 11:
/// these are not on the curated surface yet ("unproven → rejected"). They
/// publish a reference to arbitrary (possibly mutable GC) state: the atomic
/// makes the pointer swap race-free, NOT the pointee, so treating them as
/// class 2 let two tasks write the same map through `p.load()`.
pub(crate) fn is_unproven_atomic(name: &str) -> bool {
    name.starts_with("atomic.") && !is_scalar_atomic(name)
}

/// Classify a dotted foreign type name; `None` = not a known foreign type
/// (the caller falls back to local struct decls / `Unknown`).
pub(crate) fn foreign_named_type(name: &str) -> Option<TypeShape> {
    if is_scalar_atomic(name) {
        return Some(TypeShape::Atomic);
    }
    if is_unproven_atomic(name) {
        // fail-closed (O4.9): Unknown → HYB-E3005 at spawn/send boundaries,
        // with the row-11 explanation (sendability.rs)
        return Some(TypeShape::Unknown);
    }
    match name {
        "serde_json.value" => Some(TypeShape::OwnHandle("serde_json.value".into())),
        "serde_json.view" => Some(TypeShape::BorrowHandle("serde_json.view".into())),
        "flate2.writer" => Some(TypeShape::OwnHandle("flate2.writer".into())),
        _ => {
            let root = name.split('.').next().unwrap_or("");
            if BUILTIN_PACKAGES.contains(&root) {
                Some(TypeShape::ForeignOpaque(name.to_string()))
            } else {
                None
            }
        }
    }
}

/// The foreign handles the emitter ranges as element sources — the
/// `ForeignTy` range branch in `hy-emit`'s statement emission, keyed by the
/// same descriptors as the emitter's SHAPES table. `serde_json.value` is the
/// element-source handle (corpus 05 ranges `doc.array(...)` views and the
/// emitter supports ranging the value handle itself); other handles (e.g.
/// `flate2.writer`) are NOT iterable and must stay rejected (issue #411).
pub(crate) fn is_iterable_handle(name: &str) -> bool {
    name == "serde_json.value"
}

/// Mode of a package-level call: `serde_json.parse(...)`, `fmt.println(...)`.
pub(crate) fn package_mode(pkg: &str, method: &str) -> CallMode {
    match (pkg, method) {
        ("serde_json", "parse") | ("serde_json", "new_object") | ("flate2", "new_writer") => {
            CallMode::Produce
        }
        ("serde_json", "to_string_pretty")
        | ("serde_json", "array")
        | ("serde_json", "get")
        | ("serde_json", "int")
        | ("serde_json", "string") => CallMode::Borrow,
        ("serde_json", "set") | ("serde_json", "remove") => CallMode::Mutate,
        ("flate2", "write") => CallMode::Mutate,
        ("flate2", "finish") => CallMode::Consume,
        _ => CallMode::Safe,
    }
}

/// The argument index a package-level call mutates IN PLACE (T22: passing a
/// `const` root there is a write, `HYB-E3006`). `None` = the descriptor stubs
/// record no in-place write for this API.
pub(crate) fn package_mutated_arg(pkg: &str, method: &str) -> Option<usize> {
    match (pkg, method) {
        (
            "sort",
            "ints" | "strings" | "float64s" | "slice" | "slice_stable" | "sort" | "stable"
            | "Ints" | "Strings" | "Float64s" | "Slice" | "SliceStable" | "Sort" | "Stable",
        ) => Some(0),
        (
            "io",
            "read_full" | "read_at_least" | "ReadFull" | "ReadAtLeast",
        ) => Some(1),
        ("io", "copy_buffer" | "CopyBuffer") => Some(2),
        _ => None,
    }
}

/// Result shape of a package-level call (for defines / loop variables).
pub(crate) fn package_result(pkg: &str, method: &str) -> TypeShape {
    match (pkg, method) {
        ("serde_json", "parse") | ("serde_json", "new_object") => {
            TypeShape::OwnHandle("serde_json.value".into())
        }
        ("flate2", "new_writer") => TypeShape::OwnHandle("flate2.writer".into()),
        ("serde_json", "array" | "get") => TypeShape::BorrowHandle("serde_json.view".into()),
        ("serde_json", "int") | ("flate2", "write") => TypeShape::Primitive,
        ("flate2", "finish") => TypeShape::Slice(Box::new(TypeShape::Primitive)),
        ("serde_json", "to_string_pretty") | ("serde_json", "string") | ("strings", "join") => {
            TypeShape::Str
        }
        ("strings", "split") | ("strings", "fields") => TypeShape::Slice(Box::new(TypeShape::Str)),
        // `time.after(d)` returns a receive-only channel (the deadline
        // pattern, example 04); the element shape is opaque here — the
        // binding still resolves as a channel so receives on it classify.
        ("time", "after") => TypeShape::Chan(Box::new(TypeShape::Unknown)),
        ("http", "new_serve_mux") => TypeShape::ForeignOpaque("http.serve_mux".into()),
        ("http", "handler") => TypeShape::ForeignOpaque("http.handler".into()),
        // `atomic.load_pointer` / `atomic.swap_pointer` return an
        // unsafe.Pointer to arbitrary state — not a primitive (row 11)
        ("atomic", "load_pointer" | "swap_pointer") => TypeShape::Unknown,
        _ if pkg == "atomic" => TypeShape::Primitive,
        _ => TypeShape::Unknown,
    }
}

/// Static result count of a foreign package-level call returning multiple
/// values in Go (`(T, error)`, `(before, after, found)`, etc.) when not
/// unwrapped with `?` (#380 / T23 `HYB-E2002`).
pub(crate) fn package_result_count(pkg: &str, method: &str) -> Option<usize> {
    match (pkg, method) {
        (
            "strconv",
            "parse_int"
                | "parse_uint"
                | "parse_float"
                | "parse_bool"
                | "parse_complex"
                | "atoi"
                | "unquote"
                | "ParseInt"
                | "ParseUint"
                | "ParseFloat"
                | "ParseBool"
                | "ParseComplex"
                | "Atoi"
                | "Unquote",
        ) => Some(2),
        (
            "os",
            "open"
                | "create"
                | "read_file"
                | "read_dir"
                | "getwd"
                | "user_home_dir"
                | "user_cache_dir"
                | "user_config_dir"
                | "executable"
                | "lookup_env"
                | "stat"
                | "lstat"
                | "create_temp"
                | "mkdir_temp"
                | "Open"
                | "Create"
                | "ReadFile"
                | "ReadDir"
                | "Getwd"
                | "UserHomeDir"
                | "UserCacheDir"
                | "UserConfigDir"
                | "Executable"
                | "LookupEnv"
                | "Stat"
                | "Lstat"
                | "CreateTemp"
                | "MkdirTemp",
        ) => Some(2),
        (
            "io",
            "read_all"
                | "read_full"
                | "read_at_least"
                | "copy"
                | "copy_n"
                | "copy_buffer"
                | "write_string"
                | "ReadAll"
                | "ReadFull"
                | "ReadAtLeast"
                | "Copy"
                | "CopyN"
                | "CopyBuffer"
                | "WriteString",
        ) => Some(2),
        (
            "strings",
            "cut_prefix" | "cut_suffix" | "CutPrefix" | "CutSuffix",
        ) => Some(2),
        ("strings", "cut" | "Cut") => Some(3),
        (
            "filepath",
            "abs"
                | "rel"
                | "eval_symlinks"
                | "glob"
                | "match"
                | "split"
                | "Abs"
                | "Rel"
                | "EvalSymlinks"
                | "Glob"
                | "Match"
                | "Split",
        ) => Some(2),
        ("path", "match" | "split" | "Match" | "Split") => Some(2),
        (
            "time",
            "parse"
                | "parse_duration"
                | "load_location"
                | "Parse"
                | "ParseDuration"
                | "LoadLocation",
        ) => Some(2),
        (
            "net",
            "listen"
                | "dial"
                | "resolve_tcp_addr"
                | "lookup_host"
                | "Listen"
                | "Dial"
                | "ResolveTCPAddr"
                | "LookupHost",
        ) => Some(2),
        ("net", "split_host_port" | "SplitHostPort") => Some(3),
        ("hex", "decode_string" | "DecodeString") => Some(2),
        ("regex", "compile" | "match_string" | "Compile" | "MatchString") => Some(2),
        _ => None,
    }
}

/// Mode of a method call on a typed receiver, keyed by the receiver's
/// boundary type name ("serde_json.value", "flate2.writer", "atomic", ...).
pub(crate) fn method_mode(recv_type: &str, method: &str) -> CallMode {
    if recv_type.starts_with("atomic.") {
        // atomics are safe-by-type: no loan implications (class 2)
        return CallMode::Safe;
    }
    match (recv_type, method) {
        (
            "serde_json.value" | "serde_json.view",
            "array" | "get" | "int" | "string" | "to_string_pretty",
        ) => CallMode::Borrow,
        ("serde_json.value" | "serde_json.view", "set" | "remove") => CallMode::Mutate,
        ("flate2.writer", "write") => CallMode::Mutate,
        ("flate2.writer", "finish") => CallMode::Consume,
        _ => CallMode::Unknown,
    }
}

/// Result shape of a method call on a typed receiver.
pub(crate) fn method_result(recv_type: &str, method: &str) -> TypeShape {
    match (recv_type, method) {
        ("serde_json.value" | "serde_json.view", "array" | "get") => {
            // element views: iterating the result yields borrow handles
            TypeShape::BorrowHandle("serde_json.view".into())
        }
        ("serde_json.value" | "serde_json.view", "int") => TypeShape::Primitive,
        ("serde_json.value" | "serde_json.view", "string" | "to_string_pretty") => TypeShape::Str,
        ("flate2.writer", "write") => TypeShape::Primitive,
        ("flate2.writer", "finish") => TypeShape::Slice(Box::new(TypeShape::Primitive)),
        // scalar atomics only ("atomic.int" is receiver_type_name's key for
        // TypeShape::Atomic): load/swap/add yield the scalar. A non-scalar
        // atomic's `load()` / `swap()` returns a pointer / `any` — Unknown,
        // fail-closed (GUARANTEES row 11).
        _ if recv_type == "atomic.int" || is_scalar_atomic(recv_type) => TypeShape::Primitive,
        _ => TypeShape::Unknown,
    }
}
