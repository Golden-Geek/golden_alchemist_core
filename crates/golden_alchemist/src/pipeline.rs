use crate::{
    ANodeDeclaration, ANodeInstance, ANodeRoleCapability, ANodeTypeId, ContextAxisId, PipelineCardinality,
    SignatureCtx, SurfaceItemKind, TypeBindings, TypeConstraint, ValueTypeId,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PipelineShape {
    Single {
        value_type: ValueTypeId,
    },
    ValueSet {
        item_type: ValueTypeId,
        axis: Option<ContextAxisId>,
    },
    Trigger,
    CommandIntent,
    Unknown,
}

#[derive(Clone, Copy)]
pub struct PipelineShapeCheckItem<'a> {
    pub declaration: &'a dyn ANodeDeclaration,
    pub instance: &'a ANodeInstance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PipelineShapeStep {
    pub item_index: usize,
    pub node_type: ANodeTypeId,
    pub input: PipelineShape,
    pub output: PipelineShape,
    pub cardinality: PipelineCardinality,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PipelineShapeDiagnostic {
    pub item_index: usize,
    pub node_type: ANodeTypeId,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PipelineShapeResult {
    pub final_shape: PipelineShape,
    pub steps: Vec<PipelineShapeStep>,
    pub diagnostics: Vec<PipelineShapeDiagnostic>,
}

impl PipelineShapeResult {
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

pub fn check_filter_pipeline_shapes<'a>(
    initial_shape: PipelineShape,
    items: impl IntoIterator<Item = PipelineShapeCheckItem<'a>>,
    ctx: &SignatureCtx<'_>,
) -> PipelineShapeResult {
    let mut current_shape = initial_shape;
    let mut steps = Vec::new();
    let mut diagnostics = Vec::new();

    for (item_index, item) in items.into_iter().enumerate() {
        let node_type = item.declaration.type_id();
        let Some(capability) = item
            .declaration
            .role_capabilities()
            .into_iter()
            .find(|capability| capability.role == SurfaceItemKind::Filter)
        else {
            diagnostics.push(PipelineShapeDiagnostic {
                item_index,
                node_type,
                message: "ANode is not filter-capable".to_string(),
            });
            continue;
        };

        match transition_shape(&current_shape, item, &capability, ctx) {
            Ok(output) => {
                steps.push(PipelineShapeStep {
                    item_index,
                    node_type,
                    input: current_shape,
                    output: output.clone(),
                    cardinality: capability.cardinality,
                });
                current_shape = output;
            }
            Err(message) => diagnostics.push(PipelineShapeDiagnostic {
                item_index,
                node_type,
                message,
            }),
        }
    }

    PipelineShapeResult {
        final_shape: current_shape,
        steps,
        diagnostics,
    }
}

fn transition_shape(
    input: &PipelineShape,
    item: PipelineShapeCheckItem<'_>,
    capability: &ANodeRoleCapability,
    ctx: &SignatureCtx<'_>,
) -> Result<PipelineShape, String> {
    match capability.cardinality {
        PipelineCardinality::WholeSet => Ok(input.clone()),
        PipelineCardinality::Elementwise => elementwise_shape(input, item, capability, ctx),
        PipelineCardinality::Aggregate => aggregate_shape(input, item, capability, ctx),
        PipelineCardinality::Reshape => reshape_shape(input, item, capability, ctx),
        PipelineCardinality::Expand => expand_shape(input, item, capability, ctx),
    }
}

fn elementwise_shape(
    input: &PipelineShape,
    item: PipelineShapeCheckItem<'_>,
    capability: &ANodeRoleCapability,
    ctx: &SignatureCtx<'_>,
) -> Result<PipelineShape, String> {
    match input {
        PipelineShape::Single { value_type } => Ok(PipelineShape::Single {
            value_type: output_value_type(item, capability, ctx, Some(value_type))
                .unwrap_or_else(|| value_type.clone()),
        }),
        PipelineShape::ValueSet { item_type, axis } => Ok(PipelineShape::ValueSet {
            item_type: output_value_type(item, capability, ctx, Some(item_type)).unwrap_or_else(|| item_type.clone()),
            axis: axis.clone(),
        }),
        PipelineShape::Unknown => Ok(PipelineShape::Unknown),
        PipelineShape::Trigger | PipelineShape::CommandIntent => {
            Err("elementwise filters require single values or ValueSet items".to_string())
        }
    }
}

fn aggregate_shape(
    input: &PipelineShape,
    item: PipelineShapeCheckItem<'_>,
    capability: &ANodeRoleCapability,
    ctx: &SignatureCtx<'_>,
) -> Result<PipelineShape, String> {
    match input {
        PipelineShape::Single { value_type }
        | PipelineShape::ValueSet {
            item_type: value_type, ..
        } => Ok(PipelineShape::Single {
            value_type: output_value_type(item, capability, ctx, Some(value_type))
                .unwrap_or_else(|| value_type.clone()),
        }),
        PipelineShape::Unknown => Ok(PipelineShape::Unknown),
        PipelineShape::Trigger | PipelineShape::CommandIntent => {
            Err("aggregate filters require single values or ValueSet items".to_string())
        }
    }
}

fn reshape_shape(
    input: &PipelineShape,
    item: PipelineShapeCheckItem<'_>,
    capability: &ANodeRoleCapability,
    ctx: &SignatureCtx<'_>,
) -> Result<PipelineShape, String> {
    match input {
        PipelineShape::Single { value_type }
        | PipelineShape::ValueSet {
            item_type: value_type, ..
        } => output_value_type(item, capability, ctx, Some(value_type))
            .map(|value_type| PipelineShape::Single { value_type })
            .ok_or_else(|| "reshape filters must declare a resolvable primary output shape".to_string()),
        PipelineShape::Unknown => Ok(PipelineShape::Unknown),
        PipelineShape::Trigger | PipelineShape::CommandIntent => {
            Err("reshape filters require single values or ValueSet items".to_string())
        }
    }
}

fn expand_shape(
    input: &PipelineShape,
    item: PipelineShapeCheckItem<'_>,
    capability: &ANodeRoleCapability,
    ctx: &SignatureCtx<'_>,
) -> Result<PipelineShape, String> {
    match input {
        PipelineShape::Single { value_type } => Ok(PipelineShape::ValueSet {
            item_type: output_value_type(item, capability, ctx, Some(value_type)).unwrap_or_else(|| value_type.clone()),
            axis: None,
        }),
        PipelineShape::ValueSet { .. } => Err("expand filters require an explicit single-value input".to_string()),
        PipelineShape::Unknown => Ok(PipelineShape::Unknown),
        PipelineShape::Trigger | PipelineShape::CommandIntent => {
            Err("expand filters require a single value input".to_string())
        }
    }
}

fn output_value_type(
    item: PipelineShapeCheckItem<'_>,
    capability: &ANodeRoleCapability,
    ctx: &SignatureCtx<'_>,
    fallback: Option<&ValueTypeId>,
) -> Option<ValueTypeId> {
    let output_id = capability.primary_output.as_ref()?;
    let signature = item
        .declaration
        .signature(ctx, item.instance, &item.instance.type_bindings);
    let output = signature.outputs.iter().find(|output| &output.id == output_id)?;

    let mut bindings = signature.default_bindings.clone();
    let _ = bindings.merge_from(&item.instance.type_bindings);
    resolve_constraint(&output.constraint, &bindings, fallback)
}

fn resolve_constraint(
    constraint: &TypeConstraint,
    bindings: &TypeBindings,
    fallback: Option<&ValueTypeId>,
) -> Option<ValueTypeId> {
    match constraint {
        TypeConstraint::Exact(value_type) => Some(value_type.clone()),
        TypeConstraint::Generic(variable) => bindings
            .get(variable)
            .map(|binding| binding.value_type.clone())
            .or_else(|| fallback.cloned()),
        TypeConstraint::OneOf(options) => options
            .iter()
            .find_map(|option| resolve_constraint(option, bindings, fallback))
            .or_else(|| fallback.cloned()),
        TypeConstraint::Any | TypeConstraint::Facet(_) | TypeConstraint::Primitive | TypeConstraint::NumericLike => {
            fallback.cloned()
        }
    }
}

pub fn single_shape(value_type: impl Into<ValueTypeId>) -> PipelineShape {
    PipelineShape::Single {
        value_type: value_type.into(),
    }
}

pub fn value_set_shape(item_type: impl Into<ValueTypeId>, axis: Option<ContextAxisId>) -> PipelineShape {
    PipelineShape::ValueSet {
        item_type: item_type.into(),
        axis,
    }
}
