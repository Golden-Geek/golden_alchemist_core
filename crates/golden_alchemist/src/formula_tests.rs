use crate::{
    ANodeFieldPath, ANodeInstance, ANodeTypeId, AlchemistFormula, AlchemistGraph, FormulaContextContract, FormulaId,
    FormulaPropertySchema, FormulaSurface, ManagedRegionDefinition, ManagedRegionId, ManagedRegionInstance,
    ManagedRegionInstances, ManagedRegionKind, ParamUiHints, RuntimeValue, SurfaceItem, SurfaceItemId, SurfaceItemKind,
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
        managed_regions: Vec::new(),
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

#[test]
fn managed_region_kind_roundtrips_through_json() {
    let encoded = serde_json::to_string(&ManagedRegionKind::FilterPipeline).unwrap();
    assert_eq!(encoded, "\"filter_pipeline\"");

    let decoded: ManagedRegionKind = serde_json::from_str("\"action_commands\"").unwrap();
    assert_eq!(decoded, ManagedRegionKind::ActionCommands);
}

#[test]
fn empty_managed_regions_are_instantiated_from_surface() {
    let surface = FormulaSurface {
        sections: Vec::new(),
        managed_regions: vec![
            region(
                "inputs",
                ManagedRegionKind::InputSet,
                "Inputs",
                vec![SurfaceItemKind::Input],
            ),
            region(
                "filters",
                ManagedRegionKind::FilterPipeline,
                "Filters",
                vec![SurfaceItemKind::Filter],
            ),
        ],
    };

    let instances = ManagedRegionInstances::empty_for(&surface);

    assert_eq!(instances.regions.len(), 2);
    assert_eq!(
        instances
            .regions
            .get(&ManagedRegionId::new("inputs"))
            .map(|region| region.items.len()),
        Some(0)
    );
    instances
        .validate_against(&surface)
        .expect("empty regions should be valid");
}

#[test]
fn invalid_managed_region_reference_reports_diagnostic() {
    let surface = FormulaSurface {
        sections: Vec::new(),
        managed_regions: vec![region(
            "inputs",
            ManagedRegionKind::InputSet,
            "Inputs",
            vec![SurfaceItemKind::Input],
        )],
    };
    let mut instances = ManagedRegionInstances::empty_for(&surface);
    instances.regions.insert(
        ManagedRegionId::new("missing"),
        ManagedRegionInstance {
            region_id: ManagedRegionId::new("missing"),
            items: Vec::new(),
        },
    );

    let error = instances
        .validate_against(&surface)
        .expect_err("unknown region should be rejected");

    assert!(error.to_string().contains("unknown region `missing`"));
}

fn region(
    id: &str,
    kind: ManagedRegionKind,
    label: &str,
    accepted_roles: Vec<SurfaceItemKind>,
) -> ManagedRegionDefinition {
    ManagedRegionDefinition {
        id: ManagedRegionId::new(id),
        kind,
        label: label.into(),
        input_socket: None,
        output_socket: None,
        accepted_roles,
    }
}
