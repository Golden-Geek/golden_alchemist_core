use crate::{
    ANodeFieldPath, ANodeInstance, ANodeTypeId, AlchemistFormula, AlchemistGraph, FormulaContextContract,
    FormulaFamily, FormulaId, FormulaSurface, ParamUiHints, SurfaceItem, SurfaceItemId, SurfaceItemKind,
    SurfaceSection, SurfaceSectionId, SurfaceSource, ValueTypeId, ValueTypeSpec,
};

#[test]
fn formula_instance_clones_graph_surface_and_default_bindings() {
    let mut graph = AlchemistGraph::new();
    let node = graph
        .add_node(ANodeInstance::new(ANodeTypeId::new("constant"), "Constant"))
        .unwrap();
    let surface = FormulaSurface {
        sections: vec![SurfaceSection {
            id: SurfaceSectionId::new("value"),
            label: "Value".into(),
            items: vec![SurfaceItem {
                id: SurfaceItemId::new("amount"),
                label: "Amount".into(),
                description: None,
                kind: SurfaceItemKind::Parameter,
                value_type: Some(ValueTypeSpec::Exact(ValueTypeId::new("float"))),
                ui: ParamUiHints::default(),
                binding: Some(ANodeFieldPath::new(node, "value")),
            }],
            source: SurfaceSource::Formula,
        }],
    };
    let formula = AlchemistFormula {
        id: FormulaId::new("test"),
        version: 3,
        label: "Test".into(),
        family: FormulaFamily::CustomUser,
        graph,
        surface,
        context_contract: FormulaContextContract::default(),
        migrations: Vec::new(),
    };

    let instance = formula.instantiate();

    assert_eq!(instance.formula_ref.id, formula.id);
    assert_eq!(instance.formula_ref.version, 3);
    assert_eq!(instance.family, FormulaFamily::CustomUser);
    assert_eq!(instance.graph_instance, formula.graph);
    assert_eq!(instance.surface, formula.surface);
    assert!(
        instance
            .surface_bindings
            .bindings
            .contains_key(&SurfaceItemId::new("amount"))
    );
}
