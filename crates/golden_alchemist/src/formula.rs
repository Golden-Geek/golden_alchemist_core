use indexmap::IndexMap;

use crate::{
    ANodeFieldPath, ANodeId, AlchemistGraph, ContextDimensionId, Diagnostic, FormulaId, FormulaPropertyId,
    ParamUiHints, RuntimeValue, StableRef, SurfaceContributionId, SurfaceItemId, SurfaceSectionId, ValueTypeId,
    ValueTypeSpec,
};

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FormulaRef {
    pub id: FormulaId,
    pub version: u32,
}

pub type PropertyUiHints = ParamUiHints;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FormulaPropertyDecl {
    pub id: FormulaPropertyId,
    pub label: String,
    pub description: Option<String>,
    pub value_type: ValueTypeId,
    pub default_value: RuntimeValue,
    pub ui: PropertyUiHints,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FormulaPropertySchema {
    pub properties: IndexMap<FormulaPropertyId, FormulaPropertyDecl>,
}

impl FormulaPropertySchema {
    pub fn insert(&mut self, declaration: FormulaPropertyDecl) -> Option<FormulaPropertyDecl> {
        self.properties.insert(declaration.id.clone(), declaration)
    }

    #[must_use]
    pub fn get(&self, id: &FormulaPropertyId) -> Option<&FormulaPropertyDecl> {
        self.properties.get(id)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.properties.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&FormulaPropertyId, &FormulaPropertyDecl)> {
        self.properties.iter()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FormulaContextContract {
    pub required_dimensions: Vec<ContextDimensionId>,
    pub optional_dimensions: Vec<ContextDimensionId>,
    pub accepts_additional_dimensions: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FormulaMigration {
    pub from_version: u32,
    pub to_version: u32,
    pub description: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FormulaSurface {
    pub sections: Vec<SurfaceSection>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SurfaceSection {
    pub id: SurfaceSectionId,
    pub label: String,
    pub items: Vec<SurfaceItem>,
    pub source: SurfaceSource,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SurfaceSource {
    Formula,
    ANode {
        node_id: ANodeId,
        contribution_id: SurfaceContributionId,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SurfaceItemKind {
    Parameter,
    Condition,
    Consequence,
    Input,
    Filter,
    Output,
    Action,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SurfaceItem {
    pub id: SurfaceItemId,
    pub label: String,
    pub description: Option<String>,
    pub path: Vec<String>,
    pub kind: SurfaceItemKind,
    pub value_type: Option<ValueTypeSpec>,
    pub ui: ParamUiHints,
    pub bindings: Vec<ANodeFieldPath>,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FormulaSurfaceBindings {
    pub bindings: IndexMap<SurfaceItemId, Vec<ANodeFieldPath>>,
}

impl FormulaSurfaceBindings {
    #[must_use]
    pub fn from_surface(surface: &FormulaSurface) -> Self {
        let bindings = surface
            .sections
            .iter()
            .flat_map(|section| &section.items)
            .filter(|item| !item.bindings.is_empty())
            .map(|item| (item.id.clone(), item.bindings.clone()))
            .collect();
        Self { bindings }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FormulaOverrides {
    pub values: IndexMap<SurfaceItemId, RuntimeValue>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ManagedANodeBindings {
    pub configuration_roots: IndexMap<ANodeId, StableRef>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AlchemistFormula {
    pub id: FormulaId,
    pub version: u32,
    pub label: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub graph: AlchemistGraph,
    pub properties: FormulaPropertySchema,
    pub surface: FormulaSurface,
    pub context_contract: FormulaContextContract,
    pub migrations: Vec<FormulaMigration>,
}

impl AlchemistFormula {
    #[must_use]
    pub fn instantiate(&self) -> AlchemistFormulaInstance {
        AlchemistFormulaInstance {
            formula_ref: FormulaRef {
                id: self.id.clone(),
                version: self.version,
            },
            surface_bindings: FormulaSurfaceBindings::from_surface(&self.surface),
            overrides: FormulaOverrides::default(),
            managed_bindings: ManagedANodeBindings::default(),
            diagnostics: Vec::new(),
        }
    }

    pub fn materialize(
        &self,
        instance: &AlchemistFormulaInstance,
    ) -> Result<AlchemistGraph, FormulaMaterializationError> {
        instance.require_compatible(self)?;
        let mut graph = self.graph.clone();
        for (surface_item, value) in &instance.overrides.values {
            let targets = instance
                .surface_bindings
                .bindings
                .get(surface_item)
                .ok_or_else(|| FormulaMaterializationError::MissingSurfaceBinding(surface_item.clone()))?;
            for target in targets {
                let node = graph
                    .nodes
                    .get_mut(&target.node)
                    .ok_or(FormulaMaterializationError::MissingTargetNode(target.node))?;
                node.config.set(target.field.clone(), value.clone());
            }
        }
        Ok(graph)
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AlchemistFormulaInstance {
    pub formula_ref: FormulaRef,
    pub surface_bindings: FormulaSurfaceBindings,
    pub overrides: FormulaOverrides,
    pub managed_bindings: ManagedANodeBindings,
    pub diagnostics: Vec<Diagnostic>,
}

impl AlchemistFormulaInstance {
    pub fn require_compatible(&self, formula: &AlchemistFormula) -> Result<(), FormulaMaterializationError> {
        if self.formula_ref.id != formula.id {
            return Err(FormulaMaterializationError::FormulaIdMismatch {
                expected: self.formula_ref.id.clone(),
                actual: formula.id.clone(),
            });
        }
        if self.formula_ref.version != formula.version {
            return Err(FormulaMaterializationError::FormulaVersionMismatch {
                expected: self.formula_ref.version,
                actual: formula.version,
            });
        }
        Ok(())
    }
}

#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum FormulaMaterializationError {
    #[error("formula instance references `{expected}`, but definition is `{actual}`")]
    FormulaIdMismatch { expected: FormulaId, actual: FormulaId },
    #[error("formula instance references version {expected}, but definition is version {actual}")]
    FormulaVersionMismatch { expected: u32, actual: u32 },
    #[error("formula instance override references unbound surface item `{0}`")]
    MissingSurfaceBinding(SurfaceItemId),
    #[error("formula surface binding targets missing ANode `{0}`")]
    MissingTargetNode(ANodeId),
}
