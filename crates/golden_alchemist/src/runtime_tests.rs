use std::time::Duration;

use crate::{
    ANodeInstance, ANodeTypeId, AlchemistGraph, AlchemistRuntime, CompileCtx, EvaluationCtx, InputSocketRef,
    InputValueSource, OutputSocketRef, RuntimeInputSnapshot, RuntimeRegistries, RuntimeValue, TriggerValue,
    TypeBindingSource, TypeVar, ValueTypeId, ValueTypeRegistry, compile_graph, primitive_node_registry,
};

fn node(type_id: &str) -> ANodeInstance {
    ANodeInstance::new(ANodeTypeId::new(type_id), type_id)
}

fn constant(value: RuntimeValue) -> ANodeInstance {
    let mut node = node("constant");
    node.config.set("value", value);
    node
}

fn runtime(graph: &AlchemistGraph) -> AlchemistRuntime {
    let result = compile_graph(
        graph,
        &CompileCtx {
            value_types: &ValueTypeRegistry::with_primitives(),
            nodes: &primitive_node_registry(),
        },
    );
    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    AlchemistRuntime::new(result.compiled.unwrap())
}

fn evaluate(runtime: &mut AlchemistRuntime, logical_tick: u64) -> crate::RuntimeOutput {
    let value_types = ValueTypeRegistry::with_primitives();
    let registries = RuntimeRegistries {
        value_types: &value_types,
    };
    runtime.evaluate(&EvaluationCtx {
        logical_tick,
        delta_time: Duration::from_millis(16),
        events: &[],
        inputs: &RuntimeInputSnapshot::default(),
        registries: &registries,
    })
}

#[test]
fn pure_math_graph_evaluates_in_compiled_order() {
    let mut graph = AlchemistGraph::new();
    let left = graph.add_node(constant(RuntimeValue::Float(2.0))).unwrap();
    let right = graph.add_node(constant(RuntimeValue::Float(3.0))).unwrap();
    let add = graph.add_node(node("add")).unwrap();
    graph
        .connect(OutputSocketRef::new(left, "value"), InputSocketRef::new(add, "a"))
        .unwrap();
    graph
        .connect(OutputSocketRef::new(right, "value"), InputSocketRef::new(add, "b"))
        .unwrap();
    let mut runtime = runtime(&graph);
    let add_exec = runtime
        .compiled
        .debug_map
        .exec_to_authored
        .iter()
        .position(|node| *node == add)
        .map(|index| crate::ExecNodeId::new(index as u32))
        .unwrap();

    let output = evaluate(&mut runtime, 1);

    assert!(output.diagnostics.is_empty());
    assert!(
        output
            .debug_samples
            .iter()
            .any(|sample| { sample.exec_node == add_exec && sample.value == RuntimeValue::Float(5.0) })
    );
    assert_eq!(runtime.execution_count(add_exec), 1);
}

#[test]
fn forced_float_math_coerces_vec3_input_at_runtime() {
    let mut graph = AlchemistGraph::new();
    let source = graph.add_node(constant(RuntimeValue::Vec3([2.0, 4.0, 8.0]))).unwrap();
    let mut add_node = node("add");
    add_node.forced_type_bindings.insert(
        TypeVar::new("TNumeric"),
        ValueTypeId::new("float"),
        TypeBindingSource::ForcedByUser,
    );
    let add = graph.add_node(add_node).unwrap();
    graph
        .connect(OutputSocketRef::new(source, "value"), InputSocketRef::new(add, "a"))
        .unwrap();
    let mut runtime = runtime(&graph);
    let add_exec = runtime
        .compiled
        .debug_map
        .exec_to_authored
        .iter()
        .position(|node| *node == add)
        .map(|index| crate::ExecNodeId::new(index as u32))
        .unwrap();
    let add_inputs = &runtime.compiled.exec_nodes[add_exec.index()].inputs;
    assert!(
        matches!(
            &add_inputs[0],
            InputValueSource::Converted { target_type, .. } if *target_type == ValueTypeId::new("float")
        ),
        "{add_inputs:?}"
    );

    let output = evaluate(&mut runtime, 1);

    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    assert!(
        output
            .debug_samples
            .iter()
            .any(|sample| sample.exec_node == add_exec && sample.value == RuntimeValue::Float(2.0))
    );
}

#[test]
fn edge_trigger_fires_once_and_preserves_state() {
    let mut graph = AlchemistGraph::new();
    let source = graph.add_node(constant(RuntimeValue::Bool(true))).unwrap();
    let edge = graph.add_node(node("edge")).unwrap();
    graph
        .connect(
            OutputSocketRef::new(source, "value"),
            InputSocketRef::new(edge, "value"),
        )
        .unwrap();
    let mut runtime = runtime(&graph);

    let first = evaluate(&mut runtime, 10);
    let second = evaluate(&mut runtime, 11);
    let first_trigger = first
        .debug_samples
        .iter()
        .find_map(|sample| match sample.value {
            RuntimeValue::Trigger(value) => Some(value),
            _ => None,
        })
        .unwrap();
    let second_trigger = second
        .debug_samples
        .iter()
        .find_map(|sample| match sample.value {
            RuntimeValue::Trigger(value) => Some(value),
            _ => None,
        })
        .unwrap();

    assert_eq!(
        first_trigger,
        TriggerValue {
            fired: true,
            edge_id: 1,
            logical_tick: 10,
        }
    );
    assert!(!second_trigger.fired);
}

#[test]
fn runtime_diagnostics_propagate_node_failures() {
    let mut graph = AlchemistGraph::new();
    graph.add_node(node("map_range")).unwrap();
    let mut runtime = runtime(&graph);

    let output = evaluate(&mut runtime, 1);

    assert_eq!(output.diagnostics.len(), 1);
    assert!(output.diagnostics[0].message.contains("range cannot be zero"));
}

#[test]
fn effect_node_emits_intent_without_dispatching_side_effect() {
    let mut graph = AlchemistGraph::new();
    let source = graph.add_node(constant(RuntimeValue::String("hello".into()))).unwrap();
    let log = graph.add_node(node("debug_log")).unwrap();
    graph
        .connect(OutputSocketRef::new(source, "value"), InputSocketRef::new(log, "value"))
        .unwrap();
    let mut runtime = runtime(&graph);

    let output = evaluate(&mut runtime, 7);

    assert_eq!(output.intents.len(), 1);
    assert_eq!(output.intents[0].kind.as_ref(), "debug.log");
    assert_eq!(output.intents[0].logical_tick, 7);
}
