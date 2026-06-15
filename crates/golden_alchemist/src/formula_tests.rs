use crate::{
    ANodeFieldPath, ANodeInstance, ANodeTypeId, AlchemistFormula, AlchemistGraph, FormulaContextContract, FormulaId,
    FormulaPropertySchema, FormulaSurface, ParamUiHints, RuntimeValue, SurfaceItem, SurfaceItemId, SurfaceItemKind,
    SurfaceSection, SurfaceSectionId, SurfaceSource, ValueTypeId, ValueTypeSpec,
};

#[test]
fn formula_instance_references_shared_definition_and_materializes_overrides() {
    let mut graph = AlchemistGraph::new();
    let mut constant = ANodeInstance::new(ANodeTypeId::new("constant"), "Constant");
    constant.config.set("value", RuntimeValue::Float(1.0));
    let node = graph.add_node(constant).unwrap();
    let mut second = ANodeInstance::new(ANodeTypeId::new("property"), "Amount");
    second.config.set("value", RuntimeValue::Float(1.0));
    let second_node = graph.add_node(second).unwrap();
    let surface = FormulaSurface {
        sections: vec![SurfaceSection {
            id: SurfaceSectionId::new("value"),
            label: "Value".into(),
            items: vec![SurfaceItem {
                id: SurfaceItemId::new("amount"),
                label: "Amount".into(),
                description: None,
                path: vec!["Controls".into()],
                kind: SurfaceItemKind::Parameter,
                value_type: Some(ValueTypeSpec::Exact(ValueTypeId::new("float"))),
                ui: ParamUiHints::default(),
                bindings: vec![
                    ANodeFieldPath::new(node, "value"),
                    ANodeFieldPath::new(second_node, "value"),
                ],
            }],
            source: SurfaceSource::Formula,
        }],
    };
    let formula = AlchemistFormula {
        id: FormulaId::new("test"),
        version: 3,
        label: "Test".into(),
        description: None,
        tags: Vec::new(),
        graph,
        properties: FormulaPropertySchema::default(),
        surface,
        context_contract: FormulaContextContract::default(),
        migrations: Vec::new(),
    };

    let mut instance = formula.instantiate();
    instance
        .overrides
        .values
        .insert(SurfaceItemId::new("amount"), RuntimeValue::Float(2.5));
    let materialized = formula.materialize(&instance).unwrap();

    assert_eq!(instance.formula_ref.id, formula.id);
    assert_eq!(instance.formula_ref.version, 3);
    assert!(
        instance
            .surface_bindings
            .bindings
            .contains_key(&SurfaceItemId::new("amount"))
    );
    assert_eq!(
        materialized.nodes[&node].config.get("value"),
        Some(&RuntimeValue::Float(2.5))
    );
    assert_eq!(
        materialized.nodes[&second_node].config.get("value"),
        Some(&RuntimeValue::Float(2.5))
    );
    assert_eq!(
        formula.graph.nodes[&node].config.get("value"),
        Some(&RuntimeValue::Float(1.0)),
        "materializing one Processor instance must not mutate the shared Formula"
    );
}
