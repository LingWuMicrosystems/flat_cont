//! Input-model backward compatibility: a `FlatContGraph` serialised before the
//! `entry` / `visibility` fields existed still deserialises, defaulting to
//! `entry = None` and `visibility = Private`.

use flat_cont::common::FunctionVisibility;
use flat_cont::flatcont::FlatContGraph;

#[test]
fn a_graph_without_the_new_fields_deserialises_with_defaults() {
    let json = r#"{
        "name": null,
        "static_data": [],
        "externals": [],
        "functions": [
            { "continuations": [], "entry": 0 }
        ]
    }"#;

    let graph: FlatContGraph = serde_json::from_str(json).expect("old-shape graph deserialises");

    assert_eq!(graph.entry, None);
    assert_eq!(graph.functions.len(), 1);
    assert_eq!(graph.functions[0].visibility, FunctionVisibility::Private);
}

#[test]
fn function_visibility_defaults_to_private() {
    assert_eq!(FunctionVisibility::default(), FunctionVisibility::Private);
}

#[test]
fn the_new_fields_round_trip_when_present() {
    let json = r#"{
        "name": "m",
        "entry": 0,
        "static_data": [],
        "externals": [],
        "functions": [
            { "visibility": { "Public": "main" }, "continuations": [], "entry": 0 }
        ]
    }"#;

    let graph: FlatContGraph = serde_json::from_str(json).expect("round-trip graph deserialises");
    assert_eq!(graph.entry.map(|fun| fun.0), Some(0));
    assert_eq!(
        graph.functions[0].visibility,
        FunctionVisibility::Public(String::from("main"))
    );
}
