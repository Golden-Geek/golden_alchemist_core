//! Reusable typed graph compilation and runtime primitives for Golden applications.

pub mod compile;
pub mod diagnostics;
pub mod expose;
pub mod formula;
pub mod graph;
pub mod ids;
pub mod library;
pub mod node;
pub mod registry;
pub mod runtime;
#[cfg(feature = "serde")]
pub mod serialize;
pub mod typing;
pub mod value;

pub use diagnostics::{Diagnostic, DiagnosticOrigin, DiagnosticSeverity};
pub use expose::{
    ANodeFieldPath, ExposedAction, ExposedInput, ExposedOutput, ExposedParam, ExposedSurface, ParamUiHints,
    ValueTypeSpec,
};
pub use formula::{
    AlchemistFormula, AlchemistFormulaInstance, FormulaContextContract, FormulaMaterializationError, FormulaMigration,
    FormulaOverrides, FormulaPropertyDecl, FormulaPropertySchema, FormulaRef, FormulaSurface, FormulaSurfaceBindings,
    ManagedANodeBindings, PropertyUiHints, SurfaceItem, SurfaceItemKind, SurfaceSection, SurfaceSource,
};
pub use graph::{
    AEdge, ANodeConfig, ANodeInstance, ANodeUiState, AlchemistGraph, GraphComment, GraphEditError, GraphGroup,
    GraphLayout, GraphMetadata, InputSocketRef, OutputSocketRef,
};
pub use ids::{
    ANodeId, ANodeTypeId, AlchemistGraphId, ContextDimensionId, ExecNodeId, ExposedDeclId, FacetId, FormulaId,
    FormulaPropertyId, FormulaPropertySlotId, SocketId, SurfaceContributionId, SurfaceItemId, SurfaceSectionId,
    ValueSlotId, ValueTypeId,
};
pub use library::{PrimitiveNodeDeclaration, PrimitiveNodeKind, primitive_node_registry, register_primitive_nodes};
pub use node::{
    ANodeConfigFieldDecl, ANodeDeclaration, ANodeSignature, ExecutionKind, InputSocketDecl, NodeStateLayout,
    OutputSocketDecl, SignatureCtx,
};
pub use registry::{
    ANodeRegistry, ConversionKind, ConversionRule, FacetDescriptor, FacetRegistry, RegistryError, ValueTypeDescriptor,
    ValueTypeRegistry, ValueTypeUiDescriptor,
};
pub use runtime::{
    AlchemistMemory, AlchemistRuntime, CompiledNodeEvaluator, DebugCaptureSink, DebugValueSample, EvaluationCtx,
    EvaluationFrame, NodeEvaluation, RuntimeContextFrame, RuntimeDiagnostic, RuntimeEvent, RuntimeInputSnapshot,
    RuntimeIntent, RuntimeOutput, RuntimePropertyFrame, RuntimePropertyFrameError, RuntimeRegistries,
    evaluate_compiled_graph, evaluate_compiled_graph_stateless,
};
pub use typing::{
    ResolvedANode, ResolvedANodeSignature, ResolvedGraph, ResolvedSocket, TypeBinding, TypeBindingConflict,
    TypeBindingSource, TypeBindings, TypeConstraint, TypeSolveCtx, TypeSolveResult, TypeVar, solve_types,
};
pub use value::{
    ColorValue, ExtensionValue, RuntimeValue, StableRef, TriggerValue, ValueComponent, ValueStorageKind,
    component_value_type,
};

/// Current authored graph schema version.
pub const ALCHEMIST_SCHEMA_VERSION: u32 = 1;

#[cfg(test)]
mod compile_tests;
#[cfg(test)]
mod formula_tests;
#[cfg(test)]
mod graph_tests;
#[cfg(test)]
mod library_tests;
#[cfg(test)]
mod runtime_tests;
#[cfg(test)]
mod serialize_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod typing_tests;
pub use compile::{
    CompileCtx, CompileResult, CompiledAlchemistFormula, CompiledAlchemistGraph, CompiledExecNode,
    CompiledFormulaProperty, CompiledFormulaPropertySchema, CompiledNodeOperation, DebugSourceMap, DisabledOutput,
    FormulaAnalysis, FormulaCompileKey, InputValueSource, OutputRoute, RuntimeStateLayout, RuntimeSubscription,
    compile_graph,
};
