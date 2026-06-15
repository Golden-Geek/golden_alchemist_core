use crate::{
    ANodeInstance, ANodeTypeId, AlchemistGraph, CompileCtx, FormulaPropertyDecl, FormulaPropertyId,
    FormulaPropertySchema, InputSocketRef, OutputSocketRef, RuntimeValue, TypeSolveCtx, ValueTypeId, ValueTypeRegistry,
    compile_graph, primitive_node_registry, solve_types,
};

fn node(type_id: &str) -> ANodeInstance {
    ANodeInstance::new(ANodeTypeId::new(type_id), type_id)
}

fn compile(graph: &AlchemistGraph) -> crate::CompileResult {
    compile_with_properties(graph, None)
}

fn compile_with_properties(graph: &AlchemistGraph, properties: Option<&FormulaPropertySchema>) -> crate::CompileResult {
    compile_graph(
        graph,
        &CompileCtx {
            value_types: &ValueTypeRegistry::with_primitives(),
            nodes: &primitive_node_registry(),
            properties,
        },
    )
}

fn property_schema(id: &str, value_type: &str, default_value: RuntimeValue) -> FormulaPropertySchema {
    let mut schema = FormulaPropertySchema::default();
    schema.insert(FormulaPropertyDecl {
        id: FormulaPropertyId::new(id),
        label: id.into(),
        description: None,
        value_type: ValueTypeId::new(value_type),
        default_value,
        ui: crate::PropertyUiHints::default(),
    });
    schema
}

fn property_node(id: &str) -> ANodeInstance {
    let mut node = node("property");
    node.config.set("property_id", RuntimeValue::String(id.into()));
    node
}

#[test]
fn compiler_builds_dense_schedule_and_memory_layout() {
    let mut graph = AlchemistGraph::new();
    let mut constant = node("constant");
    constant.config.set("value", RuntimeValue::Float(2.0));
    let constant = graph.add_node(constant).unwrap();
    let add = graph.add_node(node("math")).unwrap();
    let delay = graph.add_node(node("delay_one_tick")).unwrap();
    graph
        .connect(
            OutputSocketRef::new(constant, "value"),
            InputSocketRef::new(add, "value1"),
        )
        .unwrap();

    let result = compile(&graph);
    let compiled = result.compiled.unwrap();

    assert_eq!(compiled.exec_nodes.len(), 3);
    assert_eq!(compiled.topo_order.len(), 3);
    assert_eq!(compiled.exec_nodes[0].exec_id.index(), 0);
    assert_eq!(compiled.exec_nodes[1].exec_id.index(), 1);
    assert_eq!(compiled.debug_map.exec_to_authored, vec![constant, add, delay]);
    assert_eq!(compiled.state_layout.state_slot_count, 1);
}

#[test]
fn cycle_without_delay_is_reported() {
    let mut graph = AlchemistGraph::new();
    let first = graph.add_node(node("math")).unwrap();
    let second = graph.add_node(node("math")).unwrap();
    graph
        .connect(
            OutputSocketRef::new(first, "result"),
            InputSocketRef::new(second, "value1"),
        )
        .unwrap();
    graph
        .connect(
            OutputSocketRef::new(second, "result"),
            InputSocketRef::new(first, "value1"),
        )
        .unwrap();

    let result = compile(&graph);

    assert!(result.compiled.is_none());
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "cycle_without_delay")
    );
}

#[test]
fn delay_node_allows_feedback() {
    let mut graph = AlchemistGraph::new();
    let add = graph.add_node(node("math")).unwrap();
    let delay = graph.add_node(node("delay_one_tick")).unwrap();
    graph
        .connect(OutputSocketRef::new(add, "result"), InputSocketRef::new(delay, "value"))
        .unwrap();
    graph
        .connect(OutputSocketRef::new(delay, "value"), InputSocketRef::new(add, "value1"))
        .unwrap();

    let result = compile(&graph);

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    assert_eq!(result.compiled.unwrap().topo_order.len(), 2);
}

#[test]
fn property_decl_rejects_invalid_default() {
    let graph = AlchemistGraph::new();
    let schema = property_schema("amount", "float", RuntimeValue::String("not a float".into()));

    let result = compile_with_properties(&graph, Some(&schema));

    assert!(result.compiled.is_none());
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "invalid_property_default_type")
    );
}

#[test]
fn property_node_rejects_missing_property_id() {
    let mut graph = AlchemistGraph::new();
    graph.add_node(node("property")).unwrap();
    let schema = property_schema("amount", "float", RuntimeValue::Float(1.0));

    let result = compile_with_properties(&graph, Some(&schema));

    assert!(result.compiled.is_none());
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "missing_property_id")
    );
}

#[test]
fn property_node_type_comes_from_schema() {
    let mut graph = AlchemistGraph::new();
    let mut property = property_node("enabled");
    property.config.set("value", RuntimeValue::Float(123.0));
    let property = graph.add_node(property).unwrap();
    let schema = property_schema("enabled", "bool", RuntimeValue::Bool(true));
    let value_types = ValueTypeRegistry::with_primitives();
    let nodes = primitive_node_registry();

    let result = solve_types(
        &graph,
        &TypeSolveCtx {
            value_types: &value_types,
            nodes: &nodes,
            properties: Some(&schema),
        },
    );

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    assert_eq!(
        result.graph.nodes[&property].signature.outputs[&crate::SocketId::new("value")].value_type,
        Some(ValueTypeId::new("bool"))
    );
}
