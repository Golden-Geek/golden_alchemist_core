use crate::{
    ANodeDeclaration, ANodeInstance, ANodeRoleCapability, ANodeSignature, ANodeTypeId, AutoWirePolicy, ExecutionKind,
    InputSocketDecl, ManagedUiMode, OutputSocketDecl, PipelineCardinality, PipelineShape, PipelineShapeCheckItem,
    PrimitiveNodeDeclaration, PrimitiveNodeKind, RuntimeValue, SignatureCtx, SocketId, SurfaceItemKind, TypeBindings,
    TypeConstraint, ValueTypeId, ValueTypeRegistry, check_filter_pipeline_shapes, single_shape, value_set_shape,
};

fn ctx(value_types: &ValueTypeRegistry) -> SignatureCtx<'_> {
    SignatureCtx {
        value_types,
        properties: None,
    }
}

fn primitive_item<'a>(
    declaration: &'a PrimitiveNodeDeclaration,
    instance: &'a ANodeInstance,
) -> PipelineShapeCheckItem<'a> {
    PipelineShapeCheckItem { declaration, instance }
}

fn primitive_instance(kind: PrimitiveNodeKind) -> (PrimitiveNodeDeclaration, ANodeInstance) {
    let declaration = PrimitiveNodeDeclaration::new(kind);
    let instance = ANodeInstance::new(declaration.type_id(), declaration.label());
    (declaration, instance)
}

#[test]
fn elementwise_filter_preserves_valueset_shape() {
    let value_types = ValueTypeRegistry::with_primitives();
    let (declaration, instance) = primitive_instance(PrimitiveNodeKind::Remap);

    let result = check_filter_pipeline_shapes(
        value_set_shape("float", Some(crate::ContextAxisId::new("input_lane"))),
        [primitive_item(&declaration, &instance)],
        &ctx(&value_types),
    );

    assert!(result.is_valid());
    assert_eq!(
        result.final_shape,
        PipelineShape::ValueSet {
            item_type: ValueTypeId::new("float"),
            axis: Some(crate::ContextAxisId::new("input_lane")),
        }
    );
    assert_eq!(result.steps[0].cardinality, PipelineCardinality::Elementwise);
}

#[test]
fn aggregate_filter_collapses_valueset_to_single() {
    let value_types = ValueTypeRegistry::with_primitives();
    let (declaration, instance) = primitive_instance(PrimitiveNodeKind::Math);

    let result = check_filter_pipeline_shapes(
        value_set_shape("float", Some(crate::ContextAxisId::new("input_lane"))),
        [primitive_item(&declaration, &instance)],
        &ctx(&value_types),
    );

    assert!(result.is_valid());
    assert_eq!(result.final_shape, single_shape("float"));
    assert_eq!(result.steps[0].cardinality, PipelineCardinality::Aggregate);
}

#[test]
fn reshape_filter_can_pack_valueset_items_to_vec3() {
    let value_types = ValueTypeRegistry::with_primitives();
    let declaration = ShapeOnlyDeclaration {
        type_name: "test_pack_vec3",
        label: "Test Pack Vec3",
        cardinality: PipelineCardinality::Reshape,
        output_type: "vec3",
    };
    let instance = ANodeInstance::new(declaration.type_id(), declaration.label());

    let result = check_filter_pipeline_shapes(
        value_set_shape("float", None),
        [PipelineShapeCheckItem {
            declaration: &declaration,
            instance: &instance,
        }],
        &ctx(&value_types),
    );

    assert!(result.is_valid());
    assert_eq!(result.final_shape, single_shape("vec3"));
    assert_eq!(result.steps[0].cardinality, PipelineCardinality::Reshape);
}

#[test]
fn checker_rejects_nodes_without_filter_capability() {
    let value_types = ValueTypeRegistry::with_primitives();
    let (declaration, instance) = primitive_instance(PrimitiveNodeKind::Constant);

    let result = check_filter_pipeline_shapes(
        single_shape("float"),
        [primitive_item(&declaration, &instance)],
        &ctx(&value_types),
    );

    assert!(!result.is_valid());
    assert_eq!(result.final_shape, single_shape("float"));
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].node_type, ANodeTypeId::new("constant"));
    assert!(result.diagnostics[0].message.contains("not filter-capable"));
}

#[test]
fn condition_gate_preserves_pipeline_shape() {
    let value_types = ValueTypeRegistry::with_primitives();
    let (declaration, instance) = primitive_instance(PrimitiveNodeKind::ConditionGate);
    let initial = value_set_shape("float", Some(crate::ContextAxisId::new("input_lane")));

    let result = check_filter_pipeline_shapes(
        initial.clone(),
        [primitive_item(&declaration, &instance)],
        &ctx(&value_types),
    );

    assert!(result.is_valid());
    assert_eq!(result.final_shape, initial);
    assert_eq!(result.steps[0].cardinality, PipelineCardinality::WholeSet);
}

#[test]
fn expand_filter_broadcasts_single_value_to_valueset() {
    let value_types = ValueTypeRegistry::with_primitives();
    let declaration = ShapeOnlyDeclaration {
        type_name: "test_broadcast",
        label: "Test Broadcast",
        cardinality: PipelineCardinality::Expand,
        output_type: "float",
    };
    let instance = ANodeInstance::new(declaration.type_id(), declaration.label());

    let result = check_filter_pipeline_shapes(
        single_shape("float"),
        [PipelineShapeCheckItem {
            declaration: &declaration,
            instance: &instance,
        }],
        &ctx(&value_types),
    );

    assert!(result.is_valid());
    assert_eq!(
        result.final_shape,
        PipelineShape::ValueSet {
            item_type: ValueTypeId::new("float"),
            axis: None,
        }
    );
    assert_eq!(result.steps[0].cardinality, PipelineCardinality::Expand);
}

struct ShapeOnlyDeclaration {
    type_name: &'static str,
    label: &'static str,
    cardinality: PipelineCardinality,
    output_type: &'static str,
}

impl ANodeDeclaration for ShapeOnlyDeclaration {
    fn type_id(&self) -> ANodeTypeId {
        ANodeTypeId::new(self.type_name)
    }

    fn label(&self) -> &'static str {
        self.label
    }

    fn category(&self) -> &'static str {
        "Test"
    }

    fn execution_kind(&self) -> ExecutionKind {
        ExecutionKind::Pure
    }

    fn role_capabilities(&self) -> Vec<ANodeRoleCapability> {
        vec![ANodeRoleCapability {
            role: SurfaceItemKind::Filter,
            primary_input: Some(SocketId::new("value")),
            primary_output: Some(SocketId::new("value")),
            autowire: AutoWirePolicy::UnaryTransform {
                input: SocketId::new("value"),
                output: SocketId::new("value"),
            },
            cardinality: self.cardinality,
            ui_mode: ManagedUiMode::CompactRow,
        }]
    }

    fn signature(
        &self,
        _ctx: &SignatureCtx<'_>,
        _instance: &ANodeInstance,
        _bindings: &TypeBindings,
    ) -> ANodeSignature {
        ANodeSignature {
            inputs: vec![
                InputSocketDecl::new("value", "Value", TypeConstraint::Any).with_default(RuntimeValue::Float(0.0)),
            ],
            outputs: vec![OutputSocketDecl::new(
                "value",
                "Value",
                TypeConstraint::Exact(ValueTypeId::new(self.output_type)),
            )],
            ..ANodeSignature::default()
        }
    }
}
