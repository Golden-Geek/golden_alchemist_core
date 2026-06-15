use std::{cmp::Ordering, fmt::Debug, sync::Arc, time::Duration};

use crate::{
    ANodeConfigFieldDecl, ANodeDeclaration, ANodeInstance, ANodeRegistry, ANodeSignature, ANodeTypeId, ColorValue,
    CompiledNodeEvaluator, CompiledNodeOperation, Diagnostic, ExecutionKind, InputSocketDecl, NodeEvaluation,
    OutputSocketDecl, RegistryError, ResolvedANodeSignature, RuntimeValue, SignatureCtx, TriggerValue,
    TypeBindingSource, TypeBindings, TypeConstraint, TypeVar, ValueTypeId,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimitiveNodeKind {
    Constant,
    Math,
    Function,
    Remap,
    SmoothFilter,
    OneMinus,
    Inverse,
    Negate,
    Speed,
    Counter,
    Lfo,
    NoiseGenerator,
    Metronome,
    CoordinateSystem,
    AngleConversion,
    GradientSampler,
    ConvertToColor,
    ExtractColor,
    Concatenate,
    ConvertToString,
    Split,
    BooleanOperation,
    Compare,
    TriggerOnOff,
    Gate,
    DelayOneTick,
    DebugValue,
    DebugLog,
}

impl PrimitiveNodeKind {
    const ALL: [Self; 28] = [
        Self::Constant,
        Self::Math,
        Self::Function,
        Self::Remap,
        Self::SmoothFilter,
        Self::OneMinus,
        Self::Inverse,
        Self::Negate,
        Self::Speed,
        Self::Counter,
        Self::Lfo,
        Self::NoiseGenerator,
        Self::Metronome,
        Self::CoordinateSystem,
        Self::AngleConversion,
        Self::GradientSampler,
        Self::ConvertToColor,
        Self::ExtractColor,
        Self::Concatenate,
        Self::ConvertToString,
        Self::Split,
        Self::BooleanOperation,
        Self::Compare,
        Self::TriggerOnOff,
        Self::Gate,
        Self::DelayOneTick,
        Self::DebugValue,
        Self::DebugLog,
    ];

    #[must_use]
    pub const fn type_name(self) -> &'static str {
        match self {
            Self::Constant => "constant",
            Self::Math => "math",
            Self::Function => "function",
            Self::Remap => "remap",
            Self::SmoothFilter => "smooth_filter",
            Self::OneMinus => "one_minus",
            Self::Inverse => "inverse",
            Self::Negate => "negate",
            Self::Speed => "speed",
            Self::Counter => "counter",
            Self::Lfo => "lfo",
            Self::NoiseGenerator => "noise_generator",
            Self::Metronome => "metronome",
            Self::CoordinateSystem => "coordinate_system",
            Self::AngleConversion => "angle_conversion",
            Self::GradientSampler => "gradient_sampler",
            Self::ConvertToColor => "convert_to_color",
            Self::ExtractColor => "extract_color",
            Self::Concatenate => "concatenate",
            Self::ConvertToString => "convert_to_string",
            Self::Split => "split",
            Self::BooleanOperation => "boolean_operation",
            Self::Compare => "compare",
            Self::TriggerOnOff => "trigger_on_off",
            Self::Gate => "gate",
            Self::DelayOneTick => "delay_one_tick",
            Self::DebugValue => "debug_value",
            Self::DebugLog => "debug_log",
        }
    }
}

pub struct PrimitiveNodeDeclaration {
    kind: PrimitiveNodeKind,
}

impl PrimitiveNodeDeclaration {
    #[must_use]
    pub const fn new(kind: PrimitiveNodeKind) -> Self {
        Self { kind }
    }

    #[must_use]
    pub const fn kind(&self) -> PrimitiveNodeKind {
        self.kind
    }
}

impl ANodeDeclaration for PrimitiveNodeDeclaration {
    fn type_id(&self) -> ANodeTypeId {
        ANodeTypeId::new(self.kind.type_name())
    }

    fn label(&self) -> &'static str {
        match self.kind {
            PrimitiveNodeKind::Constant => "Constant",
            PrimitiveNodeKind::Math => "Math",
            PrimitiveNodeKind::Function => "Function",
            PrimitiveNodeKind::Remap => "Remap",
            PrimitiveNodeKind::SmoothFilter => "Smooth Filter",
            PrimitiveNodeKind::OneMinus => "One Minus",
            PrimitiveNodeKind::Inverse => "Inverse",
            PrimitiveNodeKind::Negate => "Negate",
            PrimitiveNodeKind::Speed => "Speed",
            PrimitiveNodeKind::Counter => "Counter",
            PrimitiveNodeKind::Lfo => "LFO",
            PrimitiveNodeKind::NoiseGenerator => "Noise Generator",
            PrimitiveNodeKind::Metronome => "Metronome",
            PrimitiveNodeKind::CoordinateSystem => "Coordinate System",
            PrimitiveNodeKind::AngleConversion => "Degrees/Radians",
            PrimitiveNodeKind::GradientSampler => "Gradient Sampler",
            PrimitiveNodeKind::ConvertToColor => "Convert To Color",
            PrimitiveNodeKind::ExtractColor => "Extract Color",
            PrimitiveNodeKind::Concatenate => "Concatenate",
            PrimitiveNodeKind::ConvertToString => "Convert To String",
            PrimitiveNodeKind::Split => "Split",
            PrimitiveNodeKind::BooleanOperation => "Boolean Operation",
            PrimitiveNodeKind::Compare => "Compare",
            PrimitiveNodeKind::TriggerOnOff => "Trigger On/Off",
            PrimitiveNodeKind::Gate => "Gate",
            PrimitiveNodeKind::DelayOneTick => "Delay One Tick",
            PrimitiveNodeKind::DebugValue => "Debug Value",
            PrimitiveNodeKind::DebugLog => "Debug Log",
        }
    }

    fn category(&self) -> &'static str {
        match self.kind {
            PrimitiveNodeKind::Constant
            | PrimitiveNodeKind::Lfo
            | PrimitiveNodeKind::NoiseGenerator
            | PrimitiveNodeKind::Metronome => "Values",
            PrimitiveNodeKind::Math
            | PrimitiveNodeKind::Function
            | PrimitiveNodeKind::Remap
            | PrimitiveNodeKind::SmoothFilter
            | PrimitiveNodeKind::OneMinus
            | PrimitiveNodeKind::Inverse
            | PrimitiveNodeKind::Negate
            | PrimitiveNodeKind::Speed => "Number",
            PrimitiveNodeKind::Counter => "Number",
            PrimitiveNodeKind::CoordinateSystem | PrimitiveNodeKind::AngleConversion => "Geometry",
            PrimitiveNodeKind::GradientSampler
            | PrimitiveNodeKind::ConvertToColor
            | PrimitiveNodeKind::ExtractColor => "Color",
            PrimitiveNodeKind::Concatenate | PrimitiveNodeKind::ConvertToString | PrimitiveNodeKind::Split => "String",
            PrimitiveNodeKind::BooleanOperation | PrimitiveNodeKind::Compare => "Logic",
            PrimitiveNodeKind::TriggerOnOff | PrimitiveNodeKind::Gate | PrimitiveNodeKind::DelayOneTick => "Flow",
            PrimitiveNodeKind::DebugValue | PrimitiveNodeKind::DebugLog => "Debug",
        }
    }

    fn execution_kind(&self) -> ExecutionKind {
        match self.kind {
            PrimitiveNodeKind::SmoothFilter
            | PrimitiveNodeKind::Speed
            | PrimitiveNodeKind::Counter
            | PrimitiveNodeKind::Lfo
            | PrimitiveNodeKind::NoiseGenerator
            | PrimitiveNodeKind::Metronome
            | PrimitiveNodeKind::TriggerOnOff
            | PrimitiveNodeKind::DelayOneTick => ExecutionKind::Stateful,
            PrimitiveNodeKind::DebugLog => ExecutionKind::EffectEmitter,
            _ => ExecutionKind::Pure,
        }
    }

    fn breaks_dependency_cycle(&self) -> bool {
        self.kind == PrimitiveNodeKind::DelayOneTick
    }

    fn config_fields(&self) -> Vec<ANodeConfigFieldDecl> {
        match self.kind {
            PrimitiveNodeKind::Constant => vec![
                ANodeConfigFieldDecl::new("value", "Value", RuntimeValue::Float(0.0))
                    .with_description("The constant value emitted by this node.")
                    .with_editor("runtime_value"),
            ],
            PrimitiveNodeKind::Math => vec![
                enum_config(
                    "operator",
                    "Operator",
                    "add",
                    &[
                        ("add", "Add"),
                        ("subtract", "Subtract"),
                        ("multiply", "Multiply"),
                        ("divide", "Divide"),
                        ("modulo", "Modulo"),
                    ],
                ),
                optional_count_config("num_inputs", "Num Inputs", 2),
                value_type_config_field("TNumeric"),
            ],
            PrimitiveNodeKind::Function => vec![enum_config(
                "function",
                "Function",
                "sqrt",
                &[
                    ("sqrt", "Sqrt"),
                    ("log", "Log"),
                    ("log10", "Log10"),
                    ("exp", "Exp"),
                    ("abs", "Abs"),
                    ("floor", "Floor"),
                    ("ceil", "Ceil"),
                    ("round", "Round"),
                    ("sin", "Sin"),
                    ("cos", "Cos"),
                    ("tan", "Tan"),
                    ("asin", "Asin"),
                    ("acos", "Acos"),
                    ("atan", "Atan"),
                    ("atan2", "Atan2"),
                ],
            )],
            PrimitiveNodeKind::SmoothFilter => vec![smooth_method_config()],
            PrimitiveNodeKind::OneMinus | PrimitiveNodeKind::Inverse | PrimitiveNodeKind::Negate => {
                vec![value_type_config_field("TNumeric")]
            }
            PrimitiveNodeKind::Speed => vec![
                ANodeConfigFieldDecl::new("window_seconds", "Window", RuntimeValue::Float(0.1))
                    .with_description("Smoothing window in seconds for the speed estimate."),
            ],
            PrimitiveNodeKind::Lfo => vec![
                lfo_shape_config(),
                ANodeConfigFieldDecl::new("frequency", "Frequency", RuntimeValue::Float(1.0)),
                ANodeConfigFieldDecl::new("update_rate", "Update Rate", RuntimeValue::Float(60.0)),
                ANodeConfigFieldDecl::new("minimum", "Minimum", RuntimeValue::Float(0.0)),
                ANodeConfigFieldDecl::new("maximum", "Maximum", RuntimeValue::Float(1.0)),
            ],
            PrimitiveNodeKind::NoiseGenerator => noise_config_fields("random"),
            PrimitiveNodeKind::Metronome => vec![
                metronome_mode_config(),
                ANodeConfigFieldDecl::new("value", "Value", RuntimeValue::Float(120.0)),
                ANodeConfigFieldDecl::new("on_ratio", "On Ratio", RuntimeValue::Float(0.5)),
                ANodeConfigFieldDecl::new("randomness", "Randomness", RuntimeValue::Float(0.0)),
            ],
            PrimitiveNodeKind::CoordinateSystem => vec![enum_config(
                "mode",
                "Mode",
                "cartesian_to_polar",
                &[
                    ("cartesian_to_polar", "Cartesian To Polar"),
                    ("polar_to_cartesian", "Polar To Cartesian"),
                ],
            )],
            PrimitiveNodeKind::AngleConversion => vec![enum_config(
                "mode",
                "Mode",
                "degrees_to_radians",
                &[
                    ("degrees_to_radians", "Degrees To Radians"),
                    ("radians_to_degrees", "Radians To Degrees"),
                ],
            )],
            PrimitiveNodeKind::GradientSampler => vec![
                ANodeConfigFieldDecl::new(
                    "gradient",
                    "Gradient",
                    RuntimeValue::String(Arc::from("#000000 0, #ffffff 1")),
                )
                .with_description("Comma-separated color stops, for example `#000000 0, #ffffff 1`."),
            ],
            PrimitiveNodeKind::ConvertToColor | PrimitiveNodeKind::ExtractColor => {
                vec![color_mode_config()]
            }
            PrimitiveNodeKind::Concatenate => vec![
                optional_count_config("num_inputs", "Num Inputs", 2),
                ANodeConfigFieldDecl::new("prefix", "Prefix", RuntimeValue::String(Arc::from(""))),
                ANodeConfigFieldDecl::new("suffix", "Suffix", RuntimeValue::String(Arc::from(""))),
                ANodeConfigFieldDecl::new("separator", "Separator", RuntimeValue::String(Arc::from(""))),
            ],
            PrimitiveNodeKind::ConvertToString => vec![string_format_config()],
            PrimitiveNodeKind::Split => vec![
                ANodeConfigFieldDecl::new("separator", "Separator", RuntimeValue::String(Arc::from(","))),
                ANodeConfigFieldDecl::new("trim", "Trim", RuntimeValue::Bool(true)),
                ANodeConfigFieldDecl::new("omit_empty", "Omit Empty", RuntimeValue::Bool(false)),
            ],
            PrimitiveNodeKind::BooleanOperation => vec![enum_config(
                "operator",
                "Operator",
                "and",
                &[("and", "AND"), ("or", "OR"), ("xor", "XOR")],
            )],
            PrimitiveNodeKind::Compare => vec![
                enum_config(
                    "comparator",
                    "Comparator",
                    "equal",
                    &[
                        ("equal", "Equal"),
                        ("not_equal", "Not Equal"),
                        ("greater", "Greater"),
                        ("greater_or_equal", "Greater Or Equal"),
                        ("less", "Less"),
                        ("less_or_equal", "Less Or Equal"),
                        ("longer", "Longer"),
                        ("shorter", "Shorter"),
                        ("contains", "Contains"),
                        ("brighter", "Brighter"),
                        ("darker", "Darker"),
                    ],
                ),
                value_type_config_field_with_constraint("TValue", TypeConstraint::Primitive),
            ],
            PrimitiveNodeKind::TriggerOnOff => vec![
                ANodeConfigFieldDecl::new("toggle", "Toggle", RuntimeValue::Bool(false))
                    .with_description("Alternate On and Off triggers on rising input edges."),
            ],
            _ => Vec::new(),
        }
    }

    fn config_fields_for(&self, instance: &ANodeInstance) -> Vec<ANodeConfigFieldDecl> {
        match self.kind {
            PrimitiveNodeKind::SmoothFilter => {
                let mut fields = vec![smooth_method_config()];
                fields.extend(match config_string(instance, "method", "one_euro").as_str() {
                    "sma" | "savitzky_golay" | "median" => vec![
                        ANodeConfigFieldDecl::new("window", "Window", RuntimeValue::Int(5))
                            .with_description("Number of samples retained by the filter."),
                    ],
                    "damping" => vec![
                        ANodeConfigFieldDecl::new("mass", "Mass", RuntimeValue::Float(1.0)),
                        ANodeConfigFieldDecl::new("friction", "Friction", RuntimeValue::Float(8.0)),
                    ],
                    _ => vec![
                        ANodeConfigFieldDecl::new("min_cutoff", "Min Cutoff", RuntimeValue::Float(1.0)),
                        ANodeConfigFieldDecl::new("beta", "Beta", RuntimeValue::Float(0.0)),
                    ],
                });
                fields
            }
            PrimitiveNodeKind::ConvertToString => {
                let mut fields = vec![string_format_config()];
                match config_string(instance, "format", "decimal").as_str() {
                    "decimal" | "time" => {
                        fields.push(ANodeConfigFieldDecl::new("decimals", "Decimals", RuntimeValue::Int(3)))
                    }
                    _ => {}
                }
                fields
            }
            PrimitiveNodeKind::NoiseGenerator => noise_config_fields(&config_string(instance, "algorithm", "random")),
            _ => self.config_fields(),
        }
    }

    fn signature(&self, _ctx: &SignatureCtx<'_>, instance: &ANodeInstance, _bindings: &TypeBindings) -> ANodeSignature {
        match self.kind {
            PrimitiveNodeKind::Constant => constant_signature(instance),
            PrimitiveNodeKind::Math => {
                generic_numbered_numeric_signature("value", "Value", input_count(instance, 2), "result")
            }
            PrimitiveNodeKind::Function => function_signature(instance),
            PrimitiveNodeKind::Remap => float_signature(&["value", "in_min", "in_max", "out_min", "out_max"], "result"),
            PrimitiveNodeKind::SmoothFilter | PrimitiveNodeKind::Speed => float_signature(&["value"], "result"),
            PrimitiveNodeKind::OneMinus | PrimitiveNodeKind::Inverse | PrimitiveNodeKind::Negate => {
                generic_numeric_signature(&["value"], "result")
            }
            PrimitiveNodeKind::Counter => ANodeSignature {
                inputs: vec![
                    InputSocketDecl::new("add", "Add", exact("trigger")),
                    InputSocketDecl::new("amount", "Amount", exact("float")).with_default(RuntimeValue::Float(1.0)),
                    InputSocketDecl::new("reset", "Reset", exact("trigger")),
                ],
                outputs: vec![OutputSocketDecl::new("count", "Count", exact("float"))],
                ..ANodeSignature::default()
            },
            PrimitiveNodeKind::Lfo => ANodeSignature {
                inputs: Vec::new(),
                outputs: vec![OutputSocketDecl::new("value", "Value", exact("float"))],
                ..ANodeSignature::default()
            },
            PrimitiveNodeKind::NoiseGenerator => ANodeSignature {
                inputs: vec![
                    InputSocketDecl::new("position", "Position", exact("float")).with_default(RuntimeValue::Float(0.0)),
                ],
                outputs: vec![OutputSocketDecl::new("value", "Value", exact("float"))],
                ..ANodeSignature::default()
            },
            PrimitiveNodeKind::Metronome => ANodeSignature {
                inputs: vec![InputSocketDecl::new("tap", "Tap", exact("trigger"))],
                outputs: vec![
                    OutputSocketDecl::new("tick", "Tick", exact("trigger")),
                    OutputSocketDecl::new("gate", "Gate", exact("bool")),
                ],
                ..ANodeSignature::default()
            },
            PrimitiveNodeKind::CoordinateSystem => ANodeSignature {
                inputs: vec![InputSocketDecl::new("value", "Value", exact("vec2"))],
                outputs: vec![OutputSocketDecl::new("result", "Result", exact("vec2"))],
                ..ANodeSignature::default()
            },
            PrimitiveNodeKind::AngleConversion => float_signature(&["value"], "result"),
            PrimitiveNodeKind::GradientSampler => ANodeSignature {
                inputs: vec![
                    InputSocketDecl::new("position", "Position", exact("float")).with_default(RuntimeValue::Float(0.0)),
                ],
                outputs: vec![OutputSocketDecl::new("color", "Color", exact("color"))],
                ..ANodeSignature::default()
            },
            PrimitiveNodeKind::ConvertToColor => convert_to_color_signature(instance),
            PrimitiveNodeKind::ExtractColor => extract_color_signature(instance),
            PrimitiveNodeKind::Concatenate => ANodeSignature {
                inputs: numbered_inputs("part", "Part", input_count(instance, 2), exact("string")),
                outputs: vec![OutputSocketDecl::new("result", "Result", exact("string"))],
                ..ANodeSignature::default()
            },
            PrimitiveNodeKind::ConvertToString => ANodeSignature {
                inputs: vec![InputSocketDecl::new("value", "Value", TypeConstraint::Any)],
                outputs: vec![OutputSocketDecl::new("result", "Result", exact("string"))],
                ..ANodeSignature::default()
            },
            PrimitiveNodeKind::Split => ANodeSignature {
                inputs: vec![InputSocketDecl::new("value", "Value", exact("string"))],
                outputs: vec![OutputSocketDecl::new("values", "Values", exact("value_array"))],
                ..ANodeSignature::default()
            },
            PrimitiveNodeKind::BooleanOperation => ANodeSignature {
                inputs: vec![
                    InputSocketDecl::new("a", "A", exact("bool")),
                    InputSocketDecl::new("b", "B", exact("bool")),
                ],
                outputs: vec![OutputSocketDecl::new("result", "Result", exact("bool"))],
                ..ANodeSignature::default()
            },
            PrimitiveNodeKind::Compare => compare_signature(),
            PrimitiveNodeKind::TriggerOnOff => ANodeSignature {
                inputs: vec![InputSocketDecl::new("value", "Value", exact("bool"))],
                outputs: vec![
                    OutputSocketDecl::new("on", "On", exact("trigger")),
                    OutputSocketDecl::new("off", "Off", exact("trigger")),
                ],
                ..ANodeSignature::default()
            },
            PrimitiveNodeKind::Gate => ANodeSignature {
                inputs: vec![
                    InputSocketDecl::new("trigger", "Trigger", exact("trigger")),
                    InputSocketDecl::new("open", "Open", exact("bool")).with_default(RuntimeValue::Bool(true)),
                ],
                outputs: vec![OutputSocketDecl::new("trigger", "Trigger", exact("trigger"))],
                ..ANodeSignature::default()
            },
            PrimitiveNodeKind::DelayOneTick => passthrough_signature(),
            PrimitiveNodeKind::DebugValue => passthrough_signature(),
            PrimitiveNodeKind::DebugLog => ANodeSignature {
                inputs: vec![InputSocketDecl::new("value", "Value", TypeConstraint::Any)],
                outputs: Vec::new(),
                ..ANodeSignature::default()
            },
        }
    }

    fn compile_operation(
        &self,
        instance: &ANodeInstance,
        _resolved: &ResolvedANodeSignature,
    ) -> Result<CompiledNodeOperation, Diagnostic> {
        Ok(match self.kind {
            PrimitiveNodeKind::Constant => CompiledNodeOperation::Constant(
                instance
                    .config
                    .get("value")
                    .cloned()
                    .unwrap_or(RuntimeValue::Float(0.0)),
            ),
            PrimitiveNodeKind::Math => CompiledNodeOperation::Custom(Arc::new(MathEval {
                operator: MathOperator::from_config(instance),
            })),
            PrimitiveNodeKind::Function => CompiledNodeOperation::Custom(Arc::new(FunctionEval {
                function: FunctionKind::from_config(instance),
            })),
            PrimitiveNodeKind::Remap => CompiledNodeOperation::Custom(Arc::new(RemapEval)),
            PrimitiveNodeKind::SmoothFilter => {
                CompiledNodeOperation::Custom(Arc::new(SmoothFilterEval::from_config(instance)))
            }
            PrimitiveNodeKind::OneMinus => CompiledNodeOperation::Custom(Arc::new(UnaryNumericEval::OneMinus)),
            PrimitiveNodeKind::Inverse => CompiledNodeOperation::Custom(Arc::new(UnaryNumericEval::Inverse)),
            PrimitiveNodeKind::Negate => CompiledNodeOperation::Custom(Arc::new(UnaryNumericEval::Negate)),
            PrimitiveNodeKind::Speed => CompiledNodeOperation::Custom(Arc::new(SpeedEval {
                window_seconds: config_float(instance, "window_seconds", 0.1).max(0.0),
            })),
            PrimitiveNodeKind::Counter => CompiledNodeOperation::Custom(Arc::new(CounterEval)),
            PrimitiveNodeKind::Lfo => CompiledNodeOperation::Custom(Arc::new(LfoEval {
                shape: LfoShape::from_config(instance),
                frequency: config_float(instance, "frequency", 1.0),
                update_rate: config_float(instance, "update_rate", 60.0),
                minimum: config_float(instance, "minimum", 0.0),
                maximum: config_float(instance, "maximum", 1.0),
            })),
            PrimitiveNodeKind::NoiseGenerator => CompiledNodeOperation::Custom(Arc::new(NoiseGeneratorEval {
                algorithm: NoiseAlgorithm::from_config(instance),
                scale: config_float(instance, "scale", 1.0).max(0.0001),
                seed: config_int(instance, "seed", 0),
                octaves: config_int(instance, "octaves", 4).clamp(1, 12) as usize,
                persistence: config_float(instance, "persistence", 0.5).clamp(0.0, 1.0),
                lacunarity: config_float(instance, "lacunarity", 2.0).max(0.0001),
                jitter: config_float(instance, "jitter", 1.0).clamp(0.0, 1.0),
            })),
            PrimitiveNodeKind::Metronome => CompiledNodeOperation::Custom(Arc::new(MetronomeEval {
                mode: MetronomeMode::from_config(instance),
                value: config_float(instance, "value", 120.0),
                on_ratio: config_float(instance, "on_ratio", 0.5).clamp(0.0, 1.0),
                randomness: config_float(instance, "randomness", 0.0).clamp(0.0, 1.0),
            })),
            PrimitiveNodeKind::CoordinateSystem => CompiledNodeOperation::Custom(Arc::new(CoordinateSystemEval {
                mode: CoordinateMode::from_config(instance),
            })),
            PrimitiveNodeKind::AngleConversion => CompiledNodeOperation::Custom(Arc::new(AngleConversionEval {
                mode: AngleMode::from_config(instance),
            })),
            PrimitiveNodeKind::GradientSampler => CompiledNodeOperation::Custom(Arc::new(GradientSamplerEval {
                stops: parse_gradient(&config_string(instance, "gradient", "#000000 0, #ffffff 1")),
            })),
            PrimitiveNodeKind::ConvertToColor => CompiledNodeOperation::Custom(Arc::new(ConvertToColorEval {
                mode: ColorMode::from_config(instance),
            })),
            PrimitiveNodeKind::ExtractColor => CompiledNodeOperation::Custom(Arc::new(ExtractColorEval {
                mode: ColorMode::from_config(instance),
            })),
            PrimitiveNodeKind::Concatenate => CompiledNodeOperation::Custom(Arc::new(ConcatenateEval {
                prefix: config_string(instance, "prefix", ""),
                suffix: config_string(instance, "suffix", ""),
                separator: config_string(instance, "separator", ""),
            })),
            PrimitiveNodeKind::ConvertToString => CompiledNodeOperation::Custom(Arc::new(ConvertToStringEval {
                format: StringFormat::from_config(instance),
                decimals: config_int(instance, "decimals", 3).clamp(0, 12) as usize,
            })),
            PrimitiveNodeKind::Split => CompiledNodeOperation::Custom(Arc::new(SplitEval {
                separator: config_string(instance, "separator", ","),
                trim: config_bool(instance, "trim", true),
                omit_empty: config_bool(instance, "omit_empty", false),
            })),
            PrimitiveNodeKind::BooleanOperation => CompiledNodeOperation::Custom(Arc::new(BooleanOperationEval {
                operator: BooleanOperator::from_config(instance),
            })),
            PrimitiveNodeKind::Compare => CompiledNodeOperation::Custom(Arc::new(CompareEval {
                comparator: Comparator::from_config(instance),
            })),
            PrimitiveNodeKind::TriggerOnOff => CompiledNodeOperation::Custom(Arc::new(TriggerOnOffEval {
                toggle: config_bool(instance, "toggle", false),
            })),
            PrimitiveNodeKind::Gate => CompiledNodeOperation::Gate,
            PrimitiveNodeKind::DelayOneTick => CompiledNodeOperation::DelayOneTick,
            PrimitiveNodeKind::DebugValue => CompiledNodeOperation::Custom(Arc::new(DebugValueEval)),
            PrimitiveNodeKind::DebugLog => CompiledNodeOperation::DebugLog,
        })
    }
}

pub fn register_primitive_nodes(registry: &mut ANodeRegistry) -> Result<(), RegistryError> {
    for kind in PrimitiveNodeKind::ALL {
        registry.register(PrimitiveNodeDeclaration::new(kind))?;
    }
    Ok(())
}

#[must_use]
pub fn primitive_node_registry() -> ANodeRegistry {
    let mut registry = ANodeRegistry::default();
    register_primitive_nodes(&mut registry).expect("primitive ANode IDs must be unique");
    registry
}

fn enum_config(id: &str, label: &str, default: &str, options: &[(&str, &str)]) -> ANodeConfigFieldDecl {
    ANodeConfigFieldDecl::new(id, label, RuntimeValue::String(Arc::from(default)))
        .with_editor("enum")
        .with_enum_options(options.iter().copied())
}

fn smooth_method_config() -> ANodeConfigFieldDecl {
    enum_config(
        "method",
        "Method",
        "one_euro",
        &[
            ("one_euro", "One Euro"),
            ("sma", "SMA"),
            ("damping", "Damping"),
            ("savitzky_golay", "Savitzky-Golay"),
            ("median", "Median"),
        ],
    )
}

fn string_format_config() -> ANodeConfigFieldDecl {
    enum_config(
        "format",
        "Format",
        "decimal",
        &[
            ("decimal", "Decimal"),
            ("hexadecimal", "Hexadecimal"),
            ("time", "Time"),
            ("compact", "Compact"),
        ],
    )
}

fn lfo_shape_config() -> ANodeConfigFieldDecl {
    enum_config(
        "shape",
        "Shape",
        "sine",
        &[
            ("sine", "Sine"),
            ("triangle", "Triangle"),
            ("saw", "Saw"),
            ("square", "Square"),
            ("pulse", "Pulse"),
        ],
    )
}

fn noise_algorithm_config() -> ANodeConfigFieldDecl {
    enum_config(
        "algorithm",
        "Algorithm",
        "random",
        &[
            ("random", "Random"),
            ("perlin", "Perlin"),
            ("simplex", "Simplex"),
            ("brownian", "Brownian"),
            ("cellular", "Cellular"),
            ("fractal", "Fractal"),
        ],
    )
}

fn noise_config_fields(algorithm: &str) -> Vec<ANodeConfigFieldDecl> {
    let mut fields = vec![
        noise_algorithm_config(),
        ANodeConfigFieldDecl::new("seed", "Seed", RuntimeValue::Int(0)),
        ANodeConfigFieldDecl::new("scale", "Scale", RuntimeValue::Float(1.0)),
    ];
    match algorithm {
        "brownian" | "fractal" => fields.extend([
            ANodeConfigFieldDecl::new("octaves", "Octaves", RuntimeValue::Int(4)),
            ANodeConfigFieldDecl::new("persistence", "Persistence", RuntimeValue::Float(0.5)),
            ANodeConfigFieldDecl::new("lacunarity", "Lacunarity", RuntimeValue::Float(2.0)),
        ]),
        "cellular" => fields.push(ANodeConfigFieldDecl::new("jitter", "Jitter", RuntimeValue::Float(1.0))),
        _ => {}
    }
    fields
}

fn metronome_mode_config() -> ANodeConfigFieldDecl {
    enum_config(
        "mode",
        "Mode",
        "bpm",
        &[("frequency", "Frequency"), ("bpm", "BPM"), ("time", "Time")],
    )
}

fn color_mode_config() -> ANodeConfigFieldDecl {
    enum_config(
        "mode",
        "Mode",
        "rgba",
        &[("rgba", "RGBA"), ("hsva", "HSVA"), ("hsla", "HSLA"), ("cmyka", "CMYK")],
    )
}

fn optional_count_config(id: &str, label: &str, default: i64) -> ANodeConfigFieldDecl {
    ANodeConfigFieldDecl::new(id, label, RuntimeValue::Int(default))
        .with_description("Disable to grow automatically as new sockets are connected.")
        .with_editor("optional_count")
}

fn exact(id: &str) -> TypeConstraint {
    TypeConstraint::Exact(ValueTypeId::new(id))
}

fn constant_signature(instance: &ANodeInstance) -> ANodeSignature {
    let value_type = instance
        .config
        .get("value")
        .map_or_else(|| ValueTypeId::new("float"), RuntimeValue::value_type);
    ANodeSignature {
        inputs: Vec::new(),
        outputs: vec![OutputSocketDecl::new(
            "value",
            "Value",
            TypeConstraint::Exact(value_type),
        )],
        ..ANodeSignature::default()
    }
}

fn function_signature(instance: &ANodeInstance) -> ANodeSignature {
    let inputs = if FunctionKind::from_config(instance) == FunctionKind::Atan2 {
        vec![
            InputSocketDecl::new("y", "Y", exact("float")),
            InputSocketDecl::new("x", "X", exact("float")),
        ]
    } else {
        vec![InputSocketDecl::new("value", "Value", exact("float"))]
    };
    ANodeSignature {
        inputs,
        outputs: vec![OutputSocketDecl::new("result", "Result", exact("float"))],
        ..ANodeSignature::default()
    }
}

fn generic_numeric_signature(inputs: &[&str], output: &str) -> ANodeSignature {
    let variable = TypeVar::new("TNumeric");
    let mut default_bindings = TypeBindings::default();
    let mut generic_constraints = indexmap::IndexMap::new();
    default_bindings.insert(variable.clone(), ValueTypeId::new("float"), TypeBindingSource::Default);
    generic_constraints.insert(variable.clone(), TypeConstraint::NumericLike);
    ANodeSignature {
        inputs: inputs
            .iter()
            .map(|id| InputSocketDecl::new(*id, title(id), TypeConstraint::Generic(variable.clone())))
            .collect(),
        outputs: vec![OutputSocketDecl::new(
            output,
            title(output),
            TypeConstraint::Generic(variable),
        )],
        default_bindings,
        generic_constraints,
    }
}

fn generic_numbered_numeric_signature(prefix: &str, label: &str, count: usize, output: &str) -> ANodeSignature {
    let variable = TypeVar::new("TNumeric");
    let mut default_bindings = TypeBindings::default();
    let mut generic_constraints = indexmap::IndexMap::new();
    default_bindings.insert(variable.clone(), ValueTypeId::new("float"), TypeBindingSource::Default);
    generic_constraints.insert(variable.clone(), TypeConstraint::NumericLike);
    ANodeSignature {
        inputs: (1..=count)
            .map(|index| {
                InputSocketDecl::new(
                    format!("{prefix}{index}"),
                    format!("{label} {index}"),
                    TypeConstraint::Generic(variable.clone()),
                )
            })
            .collect(),
        outputs: vec![OutputSocketDecl::new(
            output,
            title(output),
            TypeConstraint::Generic(variable),
        )],
        default_bindings,
        generic_constraints,
    }
}

fn compare_signature() -> ANodeSignature {
    let variable = TypeVar::new("TValue");
    let mut default_bindings = TypeBindings::default();
    let mut generic_constraints = indexmap::IndexMap::new();
    default_bindings.insert(variable.clone(), ValueTypeId::new("float"), TypeBindingSource::Default);
    generic_constraints.insert(variable.clone(), TypeConstraint::Primitive);
    ANodeSignature {
        inputs: vec![
            InputSocketDecl::new("left", "Left", TypeConstraint::Generic(variable.clone())),
            InputSocketDecl::new("right", "Right", TypeConstraint::Generic(variable.clone())),
        ],
        outputs: vec![OutputSocketDecl::new("result", "Result", exact("bool"))],
        default_bindings,
        generic_constraints,
    }
}

fn color_channel_specs(mode: ColorMode) -> [(&'static str, &'static str, f64); 4] {
    match mode {
        ColorMode::Rgba => [("r", "R", 0.0), ("g", "G", 0.0), ("b", "B", 0.0), ("a", "A", 1.0)],
        ColorMode::Hsva => [("h", "H", 0.0), ("s", "S", 0.0), ("v", "V", 0.0), ("a", "A", 1.0)],
        ColorMode::Hsla => [("h", "H", 0.0), ("s", "S", 0.0), ("l", "L", 0.0), ("a", "A", 1.0)],
        ColorMode::Cmyk => [("c", "C", 0.0), ("m", "M", 0.0), ("y", "Y", 0.0), ("k", "K", 0.0)],
    }
}

fn convert_to_color_signature(instance: &ANodeInstance) -> ANodeSignature {
    let mode = ColorMode::from_config(instance);
    ANodeSignature {
        inputs: color_channel_specs(mode)
            .into_iter()
            .map(|(id, label, default)| {
                InputSocketDecl::new(id, label, exact("float")).with_default(RuntimeValue::Float(default))
            })
            .collect(),
        outputs: vec![OutputSocketDecl::new("color", "Color", exact("color"))],
        ..ANodeSignature::default()
    }
}

fn extract_color_signature(instance: &ANodeInstance) -> ANodeSignature {
    let mode = ColorMode::from_config(instance);
    ANodeSignature {
        inputs: vec![InputSocketDecl::new("color", "Color", exact("color"))],
        outputs: color_channel_specs(mode)
            .into_iter()
            .map(|(id, label, _)| OutputSocketDecl::new(id, label, exact("float")))
            .collect(),
        ..ANodeSignature::default()
    }
}

fn float_signature(inputs: &[&str], output: &str) -> ANodeSignature {
    ANodeSignature {
        inputs: inputs
            .iter()
            .map(|id| InputSocketDecl::new(*id, title(id), exact("float")))
            .collect(),
        outputs: vec![OutputSocketDecl::new(output, title(output), exact("float"))],
        ..ANodeSignature::default()
    }
}

fn numbered_inputs(prefix: &str, label: &str, count: usize, constraint: TypeConstraint) -> Vec<InputSocketDecl> {
    (1..=count)
        .map(|index| {
            InputSocketDecl::new(
                format!("{prefix}{index}"),
                format!("{label} {index}"),
                constraint.clone(),
            )
        })
        .collect()
}

fn passthrough_signature() -> ANodeSignature {
    let variable = TypeVar::new("TValue");
    let mut default_bindings = TypeBindings::default();
    default_bindings.insert(variable.clone(), ValueTypeId::new("float"), TypeBindingSource::Default);
    ANodeSignature {
        inputs: vec![InputSocketDecl::new(
            "value",
            "Value",
            TypeConstraint::Generic(variable.clone()),
        )],
        outputs: vec![OutputSocketDecl::new(
            "value",
            "Value",
            TypeConstraint::Generic(variable),
        )],
        default_bindings,
        ..ANodeSignature::default()
    }
}

fn value_type_config_field(variable: &str) -> ANodeConfigFieldDecl {
    value_type_config_field_with_constraint(variable, TypeConstraint::NumericLike)
}

fn value_type_config_field_with_constraint(variable: &str, constraint: TypeConstraint) -> ANodeConfigFieldDecl {
    ANodeConfigFieldDecl::new("value_type", "Value Type", RuntimeValue::String("float".into()))
        .with_description("Optional fixed value type for this node. Disable to infer from inputs.")
        .with_editor("value_type")
        .with_type_variable(variable)
        .with_type_options(match constraint {
            TypeConstraint::NumericLike => vec![
                ValueTypeId::new("int"),
                ValueTypeId::new("float"),
                ValueTypeId::new("vec2"),
                ValueTypeId::new("vec3"),
                ValueTypeId::new("color"),
            ],
            TypeConstraint::Primitive => vec![
                ValueTypeId::new("unit"),
                ValueTypeId::new("bool"),
                ValueTypeId::new("trigger"),
                ValueTypeId::new("int"),
                ValueTypeId::new("float"),
                ValueTypeId::new("string"),
                ValueTypeId::new("vec2"),
                ValueTypeId::new("vec3"),
                ValueTypeId::new("color"),
                ValueTypeId::new("duration"),
                ValueTypeId::new("value_array"),
            ],
            _ => Vec::new(),
        })
}

fn title(id: &str) -> String {
    id.split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn config_string(instance: &ANodeInstance, field: &str, fallback: &str) -> String {
    match instance.config.get(field) {
        Some(RuntimeValue::String(value)) => value.to_string(),
        _ => fallback.to_owned(),
    }
}

fn config_int(instance: &ANodeInstance, field: &str, fallback: i64) -> i64 {
    match instance.config.get(field) {
        Some(RuntimeValue::Int(value)) => *value,
        Some(RuntimeValue::Float(value)) => *value as i64,
        _ => fallback,
    }
}

fn config_float(instance: &ANodeInstance, field: &str, fallback: f64) -> f64 {
    match instance.config.get(field) {
        Some(RuntimeValue::Float(value)) => *value,
        Some(RuntimeValue::Int(value)) => *value as f64,
        _ => fallback,
    }
}

fn config_bool(instance: &ANodeInstance, field: &str, fallback: bool) -> bool {
    match instance.config.get(field) {
        Some(RuntimeValue::Bool(value)) => *value,
        _ => fallback,
    }
}

fn input_count(instance: &ANodeInstance, fallback: usize) -> usize {
    config_int(instance, "num_inputs", fallback as i64).clamp(1, 64) as usize
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MathOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

impl MathOperator {
    fn from_config(instance: &ANodeInstance) -> Self {
        match config_string(instance, "operator", "add").as_str() {
            "subtract" => Self::Subtract,
            "multiply" => Self::Multiply,
            "divide" => Self::Divide,
            "modulo" => Self::Modulo,
            _ => Self::Add,
        }
    }
}

#[derive(Debug)]
struct MathEval {
    operator: MathOperator,
}

impl CompiledNodeEvaluator for MathEval {
    fn evaluate(&self, evaluation: &mut NodeEvaluation<'_, '_>) -> Result<Vec<RuntimeValue>, String> {
        let Some((first, rest)) = evaluation.inputs.split_first() else {
            return Err("Math expects at least one input".into());
        };
        let mut value = first.clone();
        for next in rest {
            value = numeric_binary(&value, next, self.operator)?;
        }
        Ok(vec![value])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FunctionKind {
    Sqrt,
    Log,
    Log10,
    Exp,
    Abs,
    Floor,
    Ceil,
    Round,
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Atan2,
}

impl FunctionKind {
    fn from_config(instance: &ANodeInstance) -> Self {
        match config_string(instance, "function", "sqrt").as_str() {
            "log" => Self::Log,
            "log10" => Self::Log10,
            "exp" => Self::Exp,
            "abs" => Self::Abs,
            "floor" => Self::Floor,
            "ceil" => Self::Ceil,
            "round" => Self::Round,
            "sin" => Self::Sin,
            "cos" => Self::Cos,
            "tan" => Self::Tan,
            "asin" => Self::Asin,
            "acos" => Self::Acos,
            "atan" => Self::Atan,
            "atan2" => Self::Atan2,
            _ => Self::Sqrt,
        }
    }
}

#[derive(Debug)]
struct FunctionEval {
    function: FunctionKind,
}

impl CompiledNodeEvaluator for FunctionEval {
    fn evaluate(&self, evaluation: &mut NodeEvaluation<'_, '_>) -> Result<Vec<RuntimeValue>, String> {
        let result = if self.function == FunctionKind::Atan2 {
            let [y, x] = float_inputs::<2>(evaluation.inputs)?;
            y.atan2(x)
        } else {
            let [value] = float_inputs::<1>(evaluation.inputs)?;
            match self.function {
                FunctionKind::Sqrt => value.sqrt(),
                FunctionKind::Log => value.ln(),
                FunctionKind::Log10 => value.log10(),
                FunctionKind::Exp => value.exp(),
                FunctionKind::Abs => value.abs(),
                FunctionKind::Floor => value.floor(),
                FunctionKind::Ceil => value.ceil(),
                FunctionKind::Round => value.round(),
                FunctionKind::Sin => value.sin(),
                FunctionKind::Cos => value.cos(),
                FunctionKind::Tan => value.tan(),
                FunctionKind::Asin => value.asin(),
                FunctionKind::Acos => value.acos(),
                FunctionKind::Atan => value.atan(),
                FunctionKind::Atan2 => unreachable!(),
            }
        };
        Ok(vec![RuntimeValue::Float(result)])
    }
}

#[derive(Debug)]
struct RemapEval;

impl CompiledNodeEvaluator for RemapEval {
    fn evaluate(&self, evaluation: &mut NodeEvaluation<'_, '_>) -> Result<Vec<RuntimeValue>, String> {
        let [value, in_min, in_max, out_min, out_max] = float_inputs::<5>(evaluation.inputs)?;
        if (in_max - in_min).abs() <= f64::EPSILON {
            return Err("Remap input range cannot be zero".into());
        }
        let normalized = (value - in_min) / (in_max - in_min);
        Ok(vec![RuntimeValue::Float(out_min + normalized * (out_max - out_min))])
    }
}

#[derive(Debug)]
enum UnaryNumericEval {
    OneMinus,
    Inverse,
    Negate,
}

impl CompiledNodeEvaluator for UnaryNumericEval {
    fn evaluate(&self, evaluation: &mut NodeEvaluation<'_, '_>) -> Result<Vec<RuntimeValue>, String> {
        let Some(value) = evaluation.inputs.first() else {
            return Err("numeric unary node expects one input".into());
        };
        let result = match self {
            Self::OneMinus => numeric_map(value, |value| 1.0 - value),
            Self::Inverse => numeric_map_checked(value, |value| {
                if value.abs() <= f64::EPSILON {
                    Err("Inverse input cannot be zero".into())
                } else {
                    Ok(1.0 / value)
                }
            })?,
            Self::Negate => numeric_map(value, |value| -value),
        };
        Ok(vec![result])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SmoothMethod {
    OneEuro,
    Sma,
    Damping,
    SavitzkyGolay,
    Median,
}

#[derive(Debug)]
struct SmoothFilterEval {
    method: SmoothMethod,
    window: usize,
    min_cutoff: f64,
    beta: f64,
    mass: f64,
    friction: f64,
}

impl SmoothFilterEval {
    fn from_config(instance: &ANodeInstance) -> Self {
        let method = match config_string(instance, "method", "one_euro").as_str() {
            "sma" => SmoothMethod::Sma,
            "damping" => SmoothMethod::Damping,
            "savitzky_golay" => SmoothMethod::SavitzkyGolay,
            "median" => SmoothMethod::Median,
            _ => SmoothMethod::OneEuro,
        };
        Self {
            method,
            window: config_int(instance, "window", 5).clamp(1, 128) as usize,
            min_cutoff: config_float(instance, "min_cutoff", 1.0).max(0.0),
            beta: config_float(instance, "beta", 0.0).max(0.0),
            mass: config_float(instance, "mass", 1.0).max(0.001),
            friction: config_float(instance, "friction", 8.0).max(0.0),
        }
    }
}

impl CompiledNodeEvaluator for SmoothFilterEval {
    fn evaluate(&self, evaluation: &mut NodeEvaluation<'_, '_>) -> Result<Vec<RuntimeValue>, String> {
        let [input] = float_inputs::<1>(evaluation.inputs)?;
        let dt = delta_seconds(evaluation.ctx.delta_time);
        let state = evaluation.state.first_mut();
        let output = match self.method {
            SmoothMethod::OneEuro => {
                let mut values = state_values(state.as_deref(), 2);
                let derivative = (input - values[0]) / dt;
                let cutoff = self.min_cutoff + self.beta * derivative.abs();
                let alpha = smoothing_alpha(cutoff, dt);
                values[0] = input;
                values[1] += (input - values[1]) * alpha;
                let output = values[1];
                set_state_values(state, values);
                output
            }
            SmoothMethod::Sma => {
                let mut history = state_values(state.as_deref(), 0);
                history.push(input);
                trim_history(&mut history, self.window);
                let output = history.iter().sum::<f64>() / history.len() as f64;
                set_state_values(state, history);
                output
            }
            SmoothMethod::Damping => {
                let mut values = state_values(state.as_deref(), 2);
                let acceleration = (input - values[0]) / self.mass - values[1] * self.friction;
                values[1] += acceleration * dt;
                values[0] += values[1] * dt;
                let output = values[0];
                set_state_values(state, values);
                output
            }
            SmoothMethod::SavitzkyGolay => {
                let mut history = state_values(state.as_deref(), 0);
                history.push(input);
                trim_history(&mut history, self.window.max(5));
                let output = if history.len() >= 5 {
                    let start = history.len() - 5;
                    (-3.0 * history[start]
                        + 12.0 * history[start + 1]
                        + 17.0 * history[start + 2]
                        + 12.0 * history[start + 3]
                        - 3.0 * history[start + 4])
                        / 35.0
                } else {
                    history.iter().sum::<f64>() / history.len() as f64
                };
                set_state_values(state, history);
                output
            }
            SmoothMethod::Median => {
                let mut history = state_values(state.as_deref(), 0);
                history.push(input);
                trim_history(&mut history, self.window);
                let mut sorted = history.clone();
                sorted.sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));
                let midpoint = sorted.len() / 2;
                let output = if sorted.len() % 2 == 0 {
                    (sorted[midpoint - 1] + sorted[midpoint]) * 0.5
                } else {
                    sorted[midpoint]
                };
                set_state_values(state, history);
                output
            }
        };
        Ok(vec![RuntimeValue::Float(output)])
    }
}

#[derive(Debug)]
struct SpeedEval {
    window_seconds: f64,
}

impl CompiledNodeEvaluator for SpeedEval {
    fn evaluate(&self, evaluation: &mut NodeEvaluation<'_, '_>) -> Result<Vec<RuntimeValue>, String> {
        let [input] = float_inputs::<1>(evaluation.inputs)?;
        let dt = delta_seconds(evaluation.ctx.delta_time);
        let state = evaluation.state.first_mut();
        let mut values = state_values(state.as_deref(), 3);
        let output = if values[2] == 0.0 {
            values[2] = 1.0;
            0.0
        } else {
            let instant = (input - values[0]) / dt;
            let alpha = if self.window_seconds <= dt {
                1.0
            } else {
                (dt / self.window_seconds).clamp(0.0, 1.0)
            };
            values[1] += (instant - values[1]) * alpha;
            values[1]
        };
        values[0] = input;
        set_state_values(state, values);
        Ok(vec![RuntimeValue::Float(output)])
    }
}

#[derive(Debug)]
struct CounterEval;

impl CompiledNodeEvaluator for CounterEval {
    fn evaluate(&self, evaluation: &mut NodeEvaluation<'_, '_>) -> Result<Vec<RuntimeValue>, String> {
        let [add, amount, reset] = require_inputs::<3>(evaluation.inputs)?;
        let state = evaluation.state.first_mut();
        let mut values = state_values(state.as_deref(), 1);
        if trigger_fired(reset)? {
            values[0] = 0.0;
        } else if trigger_fired(add)? {
            values[0] += value_to_f64(amount);
        }
        let output = values[0];
        set_state_values(state, values);
        Ok(vec![RuntimeValue::Float(output)])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LfoShape {
    Sine,
    Triangle,
    Saw,
    Square,
    Pulse,
}

impl LfoShape {
    fn from_config(instance: &ANodeInstance) -> Self {
        match config_string(instance, "shape", "sine").as_str() {
            "triangle" => Self::Triangle,
            "saw" => Self::Saw,
            "square" => Self::Square,
            "pulse" => Self::Pulse,
            _ => Self::Sine,
        }
    }

    fn sample(self, phase: f64) -> f64 {
        match self {
            Self::Sine => (phase * std::f64::consts::TAU).sin() * 0.5 + 0.5,
            Self::Triangle => {
                if phase < 0.5 {
                    phase * 2.0
                } else {
                    (1.0 - phase) * 2.0
                }
            }
            Self::Saw => phase,
            Self::Square => f64::from(phase < 0.5),
            Self::Pulse => f64::from(phase < 0.1),
        }
    }
}

#[derive(Debug)]
struct LfoEval {
    shape: LfoShape,
    frequency: f64,
    update_rate: f64,
    minimum: f64,
    maximum: f64,
}

impl CompiledNodeEvaluator for LfoEval {
    fn evaluate(&self, evaluation: &mut NodeEvaluation<'_, '_>) -> Result<Vec<RuntimeValue>, String> {
        let dt = delta_seconds(evaluation.ctx.delta_time);
        let state = evaluation.state.first_mut();
        let mut values = state_values(state.as_deref(), 4);
        values[2] += dt;
        let interval = if self.update_rate <= 0.0 {
            0.0
        } else {
            1.0 / self.update_rate
        };
        if values[3] == 0.0 || interval == 0.0 || values[2] >= interval {
            values[0] = (values[0] + self.frequency * dt).rem_euclid(1.0);
            values[1] = self.minimum + self.shape.sample(values[0]) * (self.maximum - self.minimum);
            values[2] = 0.0;
            values[3] = 1.0;
        }
        let output = values[1];
        set_state_values(state, values);
        Ok(vec![RuntimeValue::Float(output)])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NoiseAlgorithm {
    Random,
    Perlin,
    Simplex,
    Brownian,
    Cellular,
    Fractal,
}

impl NoiseAlgorithm {
    fn from_config(instance: &ANodeInstance) -> Self {
        match config_string(instance, "algorithm", "random").as_str() {
            "perlin" => Self::Perlin,
            "simplex" => Self::Simplex,
            "brownian" => Self::Brownian,
            "cellular" => Self::Cellular,
            "fractal" => Self::Fractal,
            _ => Self::Random,
        }
    }
}

#[derive(Debug)]
struct NoiseGeneratorEval {
    algorithm: NoiseAlgorithm,
    scale: f64,
    seed: i64,
    octaves: usize,
    persistence: f64,
    lacunarity: f64,
    jitter: f64,
}

impl CompiledNodeEvaluator for NoiseGeneratorEval {
    fn evaluate(&self, evaluation: &mut NodeEvaluation<'_, '_>) -> Result<Vec<RuntimeValue>, String> {
        let [position] = float_inputs::<1>(evaluation.inputs)?;
        let dt = delta_seconds(evaluation.ctx.delta_time);
        let state = evaluation.state.first_mut();
        let mut values = state_values(state.as_deref(), 2);
        values[0] += dt;
        let x = (position + values[0]) * self.scale;
        let output = match self.algorithm {
            NoiseAlgorithm::Random => hash_noise(self.seed, evaluation.ctx.logical_tick as i64),
            NoiseAlgorithm::Perlin => gradient_noise_1d(self.seed, x),
            NoiseAlgorithm::Simplex => gradient_noise_1d(self.seed ^ 0x5EED, x * 1.309 + 19.19),
            NoiseAlgorithm::Brownian => {
                let step = fractal_noise_1d(self.seed, x, self.octaves, self.persistence, self.lacunarity);
                values[1] = (values[1] + step * dt.sqrt()).clamp(-1.0, 1.0);
                values[1]
            }
            NoiseAlgorithm::Cellular => cellular_noise_1d(self.seed, x, self.jitter),
            NoiseAlgorithm::Fractal => fractal_noise_1d(self.seed, x, self.octaves, self.persistence, self.lacunarity),
        };
        set_state_values(state, values);
        Ok(vec![RuntimeValue::Float(output.clamp(-1.0, 1.0))])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MetronomeMode {
    Frequency,
    Bpm,
    Time,
}

impl MetronomeMode {
    fn from_config(instance: &ANodeInstance) -> Self {
        match config_string(instance, "mode", "bpm").as_str() {
            "frequency" => Self::Frequency,
            "time" => Self::Time,
            _ => Self::Bpm,
        }
    }
}

#[derive(Debug)]
struct MetronomeEval {
    mode: MetronomeMode,
    value: f64,
    on_ratio: f64,
    randomness: f64,
}

impl MetronomeEval {
    fn seed_for(&self, logical_tick: u64) -> i64 {
        self.value.to_bits() as i64 ^ logical_tick as i64
    }
}

impl CompiledNodeEvaluator for MetronomeEval {
    fn evaluate(&self, evaluation: &mut NodeEvaluation<'_, '_>) -> Result<Vec<RuntimeValue>, String> {
        let [tap] = require_inputs::<1>(evaluation.inputs)?;
        let dt = delta_seconds(evaluation.ctx.delta_time);
        let state = evaluation.state.first_mut();
        let mut values = state_values(state.as_deref(), 5);
        values[1] += dt;
        if trigger_fired(tap)? {
            if values[2] > 0.0 {
                values[3] = (values[1] - values[2]).max(0.001);
                values[4] = values[3];
            }
            values[2] = values[1];
        }
        let base_period = match self.mode {
            MetronomeMode::Frequency => {
                if self.value.abs() <= f64::EPSILON {
                    1.0
                } else {
                    1.0 / self.value.abs()
                }
            }
            MetronomeMode::Bpm => {
                if self.value.abs() <= f64::EPSILON {
                    0.5
                } else {
                    60.0 / self.value.abs()
                }
            }
            MetronomeMode::Time => self.value.abs().max(0.001),
        };
        if values[4] <= 0.0 {
            values[4] = randomized_period(base_period, self.randomness, self.seed_for(evaluation.ctx.logical_tick));
        }
        let period = if values[3] > 0.0 { values[3] } else { values[4] }.max(0.001);
        values[0] += dt / period;
        let mut fired = false;
        if values[0] >= 1.0 {
            values[0] = values[0].fract();
            values[4] = randomized_period(
                base_period,
                self.randomness,
                self.seed_for(evaluation.ctx.logical_tick + 1),
            );
            fired = true;
        }
        let gate = values[0] < self.on_ratio;
        set_state_values(state, values);
        let edge_id = u64::from(evaluation.exec_node.index() as u32);
        Ok(vec![
            RuntimeValue::Trigger(trigger(fired, edge_id, evaluation.ctx.logical_tick)),
            RuntimeValue::Bool(gate),
        ])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CoordinateMode {
    CartesianToPolar,
    PolarToCartesian,
}

impl CoordinateMode {
    fn from_config(instance: &ANodeInstance) -> Self {
        match config_string(instance, "mode", "cartesian_to_polar").as_str() {
            "polar_to_cartesian" => Self::PolarToCartesian,
            _ => Self::CartesianToPolar,
        }
    }
}

#[derive(Debug)]
struct CoordinateSystemEval {
    mode: CoordinateMode,
}

impl CompiledNodeEvaluator for CoordinateSystemEval {
    fn evaluate(&self, evaluation: &mut NodeEvaluation<'_, '_>) -> Result<Vec<RuntimeValue>, String> {
        let Some(RuntimeValue::Vec2(value)) = evaluation.inputs.first() else {
            return Err("Coordinate System expects a vec2 input".into());
        };
        let result = match self.mode {
            CoordinateMode::CartesianToPolar => {
                let radius = value[0].hypot(value[1]);
                let angle = value[1].atan2(value[0]);
                [radius, angle]
            }
            CoordinateMode::PolarToCartesian => {
                let radius = value[0];
                let angle = value[1];
                [radius * angle.cos(), radius * angle.sin()]
            }
        };
        Ok(vec![RuntimeValue::Vec2(result)])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AngleMode {
    DegreesToRadians,
    RadiansToDegrees,
}

impl AngleMode {
    fn from_config(instance: &ANodeInstance) -> Self {
        match config_string(instance, "mode", "degrees_to_radians").as_str() {
            "radians_to_degrees" => Self::RadiansToDegrees,
            _ => Self::DegreesToRadians,
        }
    }
}

#[derive(Debug)]
struct AngleConversionEval {
    mode: AngleMode,
}

impl CompiledNodeEvaluator for AngleConversionEval {
    fn evaluate(&self, evaluation: &mut NodeEvaluation<'_, '_>) -> Result<Vec<RuntimeValue>, String> {
        let [value] = float_inputs::<1>(evaluation.inputs)?;
        let result = match self.mode {
            AngleMode::DegreesToRadians => value.to_radians(),
            AngleMode::RadiansToDegrees => value.to_degrees(),
        };
        Ok(vec![RuntimeValue::Float(result)])
    }
}

#[derive(Clone, Debug)]
struct GradientStop {
    position: f64,
    color: ColorValue,
}

#[derive(Debug)]
struct GradientSamplerEval {
    stops: Vec<GradientStop>,
}

impl CompiledNodeEvaluator for GradientSamplerEval {
    fn evaluate(&self, evaluation: &mut NodeEvaluation<'_, '_>) -> Result<Vec<RuntimeValue>, String> {
        let [position] = float_inputs::<1>(evaluation.inputs)?;
        Ok(vec![RuntimeValue::Color(sample_gradient(
            &self.stops,
            position.clamp(0.0, 1.0),
        ))])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ColorMode {
    Rgba,
    Hsva,
    Hsla,
    Cmyk,
}

impl ColorMode {
    fn from_config(instance: &ANodeInstance) -> Self {
        match config_string(instance, "mode", "rgba").as_str() {
            "hsva" => Self::Hsva,
            "hsla" => Self::Hsla,
            "cmyka" | "cmyk" => Self::Cmyk,
            _ => Self::Rgba,
        }
    }
}

#[derive(Debug)]
struct ConvertToColorEval {
    mode: ColorMode,
}

impl CompiledNodeEvaluator for ConvertToColorEval {
    fn evaluate(&self, evaluation: &mut NodeEvaluation<'_, '_>) -> Result<Vec<RuntimeValue>, String> {
        let [first, second, third, fourth] = float_inputs::<4>(evaluation.inputs)?;
        let color = match self.mode {
            ColorMode::Rgba => ColorValue {
                red: first as f32,
                green: second as f32,
                blue: third as f32,
                alpha: fourth as f32,
            },
            ColorMode::Hsva => hsva_to_rgba(first, second, third, fourth),
            ColorMode::Hsla => hsla_to_rgba(first, second, third, fourth),
            ColorMode::Cmyk => cmyk_to_rgba(first, second, third, fourth),
        };
        Ok(vec![RuntimeValue::Color(color)])
    }
}

#[derive(Debug)]
struct ExtractColorEval {
    mode: ColorMode,
}

impl CompiledNodeEvaluator for ExtractColorEval {
    fn evaluate(&self, evaluation: &mut NodeEvaluation<'_, '_>) -> Result<Vec<RuntimeValue>, String> {
        let Some(RuntimeValue::Color(color)) = evaluation.inputs.first() else {
            return Err("Extract Color expects a color input".into());
        };
        let channels = match self.mode {
            ColorMode::Rgba => [
                f64::from(color.red),
                f64::from(color.green),
                f64::from(color.blue),
                f64::from(color.alpha),
            ],
            ColorMode::Hsva => rgba_to_hsva(*color),
            ColorMode::Hsla => rgba_to_hsla(*color),
            ColorMode::Cmyk => rgba_to_cmyk(*color),
        };
        Ok(channels.into_iter().map(RuntimeValue::Float).collect())
    }
}

#[derive(Debug)]
struct DebugValueEval;

impl CompiledNodeEvaluator for DebugValueEval {
    fn evaluate(&self, evaluation: &mut NodeEvaluation<'_, '_>) -> Result<Vec<RuntimeValue>, String> {
        let [value] = require_inputs::<1>(evaluation.inputs)?;
        Ok(vec![value.clone()])
    }
}

#[derive(Debug)]
struct ConcatenateEval {
    prefix: String,
    suffix: String,
    separator: String,
}

impl CompiledNodeEvaluator for ConcatenateEval {
    fn evaluate(&self, evaluation: &mut NodeEvaluation<'_, '_>) -> Result<Vec<RuntimeValue>, String> {
        let body = evaluation
            .inputs
            .iter()
            .map(|value| format_runtime_value(value, 3))
            .collect::<Vec<_>>()
            .join(&self.separator);
        Ok(vec![RuntimeValue::String(Arc::from(format!(
            "{}{}{}",
            self.prefix, body, self.suffix
        )))])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StringFormat {
    Decimal,
    Hexadecimal,
    Time,
    Compact,
}

impl StringFormat {
    fn from_config(instance: &ANodeInstance) -> Self {
        match config_string(instance, "format", "decimal").as_str() {
            "hexadecimal" => Self::Hexadecimal,
            "time" => Self::Time,
            "compact" => Self::Compact,
            _ => Self::Decimal,
        }
    }
}

#[derive(Debug)]
struct ConvertToStringEval {
    format: StringFormat,
    decimals: usize,
}

impl CompiledNodeEvaluator for ConvertToStringEval {
    fn evaluate(&self, evaluation: &mut NodeEvaluation<'_, '_>) -> Result<Vec<RuntimeValue>, String> {
        let Some(value) = evaluation.inputs.first() else {
            return Err("Convert To String expects one input".into());
        };
        let text = match self.format {
            StringFormat::Decimal => decimal_string(value, self.decimals),
            StringFormat::Hexadecimal => format!("0x{:X}", value_to_i64(value)),
            StringFormat::Time => time_string(value_to_f64(value), self.decimals),
            StringFormat::Compact => format_runtime_value(value, self.decimals),
        };
        Ok(vec![RuntimeValue::String(Arc::from(text))])
    }
}

#[derive(Debug)]
struct SplitEval {
    separator: String,
    trim: bool,
    omit_empty: bool,
}

impl CompiledNodeEvaluator for SplitEval {
    fn evaluate(&self, evaluation: &mut NodeEvaluation<'_, '_>) -> Result<Vec<RuntimeValue>, String> {
        let Some(RuntimeValue::String(value)) = evaluation.inputs.first() else {
            return Err("Split expects a string input".into());
        };
        let parts: Vec<String> = if self.separator.is_empty() {
            value.chars().map(|character| character.to_string()).collect()
        } else {
            value.split(&self.separator).map(ToOwned::to_owned).collect()
        };
        let values = parts
            .into_iter()
            .map(|part| if self.trim { part.trim().to_owned() } else { part })
            .filter(|part| !self.omit_empty || !part.is_empty())
            .map(|part| RuntimeValue::String(Arc::from(part)))
            .collect();
        Ok(vec![RuntimeValue::Array(values)])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BooleanOperator {
    And,
    Or,
    Xor,
}

impl BooleanOperator {
    fn from_config(instance: &ANodeInstance) -> Self {
        match config_string(instance, "operator", "and").as_str() {
            "or" => Self::Or,
            "xor" => Self::Xor,
            _ => Self::And,
        }
    }
}

#[derive(Debug)]
struct BooleanOperationEval {
    operator: BooleanOperator,
}

impl CompiledNodeEvaluator for BooleanOperationEval {
    fn evaluate(&self, evaluation: &mut NodeEvaluation<'_, '_>) -> Result<Vec<RuntimeValue>, String> {
        let values = evaluation
            .inputs
            .iter()
            .map(runtime_bool)
            .collect::<Result<Vec<_>, _>>()?;
        let result = match self.operator {
            BooleanOperator::And => values.into_iter().all(|value| value),
            BooleanOperator::Or => values.into_iter().any(|value| value),
            BooleanOperator::Xor => values.into_iter().filter(|value| *value).count() % 2 == 1,
        };
        Ok(vec![RuntimeValue::Bool(result)])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Comparator {
    Equal,
    NotEqual,
    Greater,
    GreaterOrEqual,
    Less,
    LessOrEqual,
    Longer,
    Shorter,
    Contains,
    Brighter,
    Darker,
}

impl Comparator {
    fn from_config(instance: &ANodeInstance) -> Self {
        match config_string(instance, "comparator", "equal").as_str() {
            "not_equal" => Self::NotEqual,
            "greater" => Self::Greater,
            "greater_or_equal" => Self::GreaterOrEqual,
            "less" => Self::Less,
            "less_or_equal" => Self::LessOrEqual,
            "longer" => Self::Longer,
            "shorter" => Self::Shorter,
            "contains" => Self::Contains,
            "brighter" => Self::Brighter,
            "darker" => Self::Darker,
            _ => Self::Equal,
        }
    }
}

#[derive(Debug)]
struct CompareEval {
    comparator: Comparator,
}

impl CompiledNodeEvaluator for CompareEval {
    fn evaluate(&self, evaluation: &mut NodeEvaluation<'_, '_>) -> Result<Vec<RuntimeValue>, String> {
        let [left, right] = require_inputs::<2>(evaluation.inputs)?;
        let result = match self.comparator {
            Comparator::Equal => left == right,
            Comparator::NotEqual => left != right,
            Comparator::Greater => value_to_f64(left) > value_to_f64(right),
            Comparator::GreaterOrEqual => value_to_f64(left) >= value_to_f64(right),
            Comparator::Less => value_to_f64(left) < value_to_f64(right),
            Comparator::LessOrEqual => value_to_f64(left) <= value_to_f64(right),
            Comparator::Longer => format_runtime_value(left, 3).len() > format_runtime_value(right, 3).len(),
            Comparator::Shorter => format_runtime_value(left, 3).len() < format_runtime_value(right, 3).len(),
            Comparator::Contains => format_runtime_value(left, 3).contains(&format_runtime_value(right, 3)),
            Comparator::Brighter => brightness(left) > brightness(right),
            Comparator::Darker => brightness(left) < brightness(right),
        };
        Ok(vec![RuntimeValue::Bool(result)])
    }
}

#[derive(Debug)]
struct TriggerOnOffEval {
    toggle: bool,
}

impl CompiledNodeEvaluator for TriggerOnOffEval {
    fn evaluate(&self, evaluation: &mut NodeEvaluation<'_, '_>) -> Result<Vec<RuntimeValue>, String> {
        let [value] = bool_inputs::<1>(evaluation.inputs)?;
        let state = evaluation.state.first_mut();
        let mut values = state_values(state.as_deref(), 2);
        let previous = values[0] != 0.0;
        let rising = value && !previous;
        let falling = !value && previous;
        let mut on = false;
        let mut off = false;
        if self.toggle {
            if rising {
                let toggled_on = values[1] == 0.0;
                values[1] = f64::from(toggled_on);
                on = toggled_on;
                off = !toggled_on;
            }
        } else {
            on = rising;
            off = falling;
        }
        values[0] = f64::from(value);
        set_state_values(state, values);
        let edge_id = u64::from(evaluation.exec_node.index() as u32);
        Ok(vec![
            RuntimeValue::Trigger(trigger(on, edge_id, evaluation.ctx.logical_tick)),
            RuntimeValue::Trigger(trigger(off, edge_id, evaluation.ctx.logical_tick)),
        ])
    }
}

fn require_inputs<const N: usize>(inputs: &[RuntimeValue]) -> Result<[&RuntimeValue; N], String> {
    inputs
        .iter()
        .collect::<Vec<_>>()
        .try_into()
        .map_err(|_| format!("node expects {N} input(s)"))
}

fn bool_inputs<const N: usize>(inputs: &[RuntimeValue]) -> Result<[bool; N], String> {
    require_inputs::<N>(inputs)?
        .map(runtime_bool)
        .into_iter()
        .collect::<Result<Vec<_>, String>>()?
        .try_into()
        .map_err(|_| "invalid boolean input count".into())
}

fn float_inputs<const N: usize>(inputs: &[RuntimeValue]) -> Result<[f64; N], String> {
    require_inputs::<N>(inputs)?
        .map(|value| Ok(value_to_f64(value)))
        .into_iter()
        .collect::<Result<Vec<_>, String>>()?
        .try_into()
        .map_err(|_| "invalid numeric input count".into())
}

fn runtime_bool(value: &RuntimeValue) -> Result<bool, String> {
    match value {
        RuntimeValue::Bool(value) => Ok(*value),
        RuntimeValue::Trigger(value) => Ok(value.fired),
        _ => Err("node expects boolean inputs".into()),
    }
}

fn trigger_fired(value: &RuntimeValue) -> Result<bool, String> {
    match value {
        RuntimeValue::Trigger(value) => Ok(value.fired),
        RuntimeValue::Bool(value) => Ok(*value),
        _ => Err("node expects trigger inputs".into()),
    }
}

fn hash_u64(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

fn hash_noise(seed: i64, index: i64) -> f64 {
    let combined = (seed as u64)
        .wrapping_mul(0x9e37_79b9_7f4a_7c15)
        .wrapping_add(index as u64);
    let mantissa = hash_u64(combined) >> 11;
    let unit = mantissa as f64 * (1.0 / ((1_u64 << 53) as f64));
    unit * 2.0 - 1.0
}

fn smoothstep(value: f64) -> f64 {
    value * value * (3.0 - 2.0 * value)
}

fn lerp(left: f64, right: f64, amount: f64) -> f64 {
    left + (right - left) * amount
}

fn gradient_noise_1d(seed: i64, x: f64) -> f64 {
    let base = x.floor() as i64;
    let local = x - base as f64;
    let weight = smoothstep(local);
    let left = hash_noise(seed, base) * local;
    let right = hash_noise(seed, base + 1) * (local - 1.0);
    (lerp(left, right, weight) * 2.0).clamp(-1.0, 1.0)
}

fn fractal_noise_1d(seed: i64, x: f64, octaves: usize, persistence: f64, lacunarity: f64) -> f64 {
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut sum = 0.0;
    let mut amplitude_sum = 0.0;
    for octave in 0..octaves {
        sum += gradient_noise_1d(seed + octave as i64 * 101, x * frequency) * amplitude;
        amplitude_sum += amplitude;
        amplitude *= persistence;
        frequency *= lacunarity;
    }
    if amplitude_sum <= f64::EPSILON {
        0.0
    } else {
        (sum / amplitude_sum).clamp(-1.0, 1.0)
    }
}

fn cellular_noise_1d(seed: i64, x: f64, jitter: f64) -> f64 {
    let base = x.floor() as i64;
    let mut nearest = f64::INFINITY;
    for cell in (base - 1)..=(base + 1) {
        let random = hash_noise(seed, cell) * 0.5 + 0.5;
        let feature = cell as f64 + random * jitter;
        nearest = nearest.min((x - feature).abs());
    }
    (1.0 - nearest.clamp(0.0, 1.0) * 2.0).clamp(-1.0, 1.0)
}

fn randomized_period(period: f64, randomness: f64, seed: i64) -> f64 {
    let factor = 1.0 + hash_noise(seed, 0) * randomness;
    (period * factor.max(0.001)).max(0.001)
}

fn parse_gradient(definition: &str) -> Vec<GradientStop> {
    let parsed = definition
        .split(',')
        .filter_map(parse_gradient_stop)
        .collect::<Vec<_>>();
    if parsed.is_empty() {
        return vec![
            GradientStop {
                position: 0.0,
                color: ColorValue::BLACK,
            },
            GradientStop {
                position: 1.0,
                color: ColorValue {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 1.0,
                },
            },
        ];
    }
    let last_index = parsed.len().saturating_sub(1).max(1);
    let mut stops = parsed
        .into_iter()
        .enumerate()
        .map(|(index, (color, position))| GradientStop {
            position: position.unwrap_or(index as f64 / last_index as f64).clamp(0.0, 1.0),
            color,
        })
        .collect::<Vec<_>>();
    stops.sort_by(|left, right| left.position.partial_cmp(&right.position).unwrap_or(Ordering::Equal));
    stops
}

fn parse_gradient_stop(part: &str) -> Option<(ColorValue, Option<f64>)> {
    let mut tokens = part.split_whitespace();
    let color = parse_hex_color(tokens.next()?)?;
    let position = tokens.next().and_then(|token| token.parse::<f64>().ok());
    Some((color, position))
}

fn parse_hex_color(token: &str) -> Option<ColorValue> {
    let value = token.trim().strip_prefix('#')?;
    let expanded;
    let hex = match value.len() {
        3 | 4 => {
            expanded = value
                .chars()
                .flat_map(|character| [character, character])
                .collect::<String>();
            expanded.as_str()
        }
        6 | 8 => value,
        _ => return None,
    };
    let red = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let green = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let blue = u8::from_str_radix(&hex[4..6], 16).ok()?;
    let alpha = if hex.len() >= 8 {
        u8::from_str_radix(&hex[6..8], 16).ok()?
    } else {
        u8::MAX
    };
    Some(ColorValue {
        red: f32::from(red) / 255.0,
        green: f32::from(green) / 255.0,
        blue: f32::from(blue) / 255.0,
        alpha: f32::from(alpha) / 255.0,
    })
}

fn sample_gradient(stops: &[GradientStop], position: f64) -> ColorValue {
    let Some(first) = stops.first() else {
        return ColorValue::BLACK;
    };
    if position <= first.position {
        return first.color;
    }
    for pair in stops.windows(2) {
        let left = &pair[0];
        let right = &pair[1];
        if position > right.position {
            continue;
        }
        let width = (right.position - left.position).max(f64::EPSILON);
        let amount = ((position - left.position) / width).clamp(0.0, 1.0);
        return ColorValue {
            red: lerp(f64::from(left.color.red), f64::from(right.color.red), amount) as f32,
            green: lerp(f64::from(left.color.green), f64::from(right.color.green), amount) as f32,
            blue: lerp(f64::from(left.color.blue), f64::from(right.color.blue), amount) as f32,
            alpha: lerp(f64::from(left.color.alpha), f64::from(right.color.alpha), amount) as f32,
        };
    }
    stops.last().map_or(ColorValue::BLACK, |stop| stop.color)
}

fn clamp01(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

fn hue_sector(hue_degrees: f64, chroma: f64) -> (f64, f64, f64) {
    let hue = hue_degrees.rem_euclid(360.0) / 60.0;
    let x = chroma * (1.0 - (hue.rem_euclid(2.0) - 1.0).abs());
    match hue.floor() as i32 {
        0 => (chroma, x, 0.0),
        1 => (x, chroma, 0.0),
        2 => (0.0, chroma, x),
        3 => (0.0, x, chroma),
        4 => (x, 0.0, chroma),
        _ => (chroma, 0.0, x),
    }
}

fn hsva_to_rgba(hue: f64, saturation: f64, value: f64, alpha: f64) -> ColorValue {
    let saturation = clamp01(saturation);
    let value = clamp01(value);
    let chroma = value * saturation;
    let (red, green, blue) = hue_sector(hue, chroma);
    let m = value - chroma;
    ColorValue {
        red: (red + m) as f32,
        green: (green + m) as f32,
        blue: (blue + m) as f32,
        alpha: clamp01(alpha) as f32,
    }
}

fn hsla_to_rgba(hue: f64, saturation: f64, lightness: f64, alpha: f64) -> ColorValue {
    let saturation = clamp01(saturation);
    let lightness = clamp01(lightness);
    let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let (red, green, blue) = hue_sector(hue, chroma);
    let m = lightness - chroma * 0.5;
    ColorValue {
        red: (red + m) as f32,
        green: (green + m) as f32,
        blue: (blue + m) as f32,
        alpha: clamp01(alpha) as f32,
    }
}

fn cmyk_to_rgba(cyan: f64, magenta: f64, yellow: f64, key: f64) -> ColorValue {
    let cyan = clamp01(cyan);
    let magenta = clamp01(magenta);
    let yellow = clamp01(yellow);
    let key = clamp01(key);
    ColorValue {
        red: ((1.0 - cyan) * (1.0 - key)) as f32,
        green: ((1.0 - magenta) * (1.0 - key)) as f32,
        blue: ((1.0 - yellow) * (1.0 - key)) as f32,
        alpha: 1.0,
    }
}

fn color_hue(red: f64, green: f64, blue: f64, max: f64, delta: f64) -> f64 {
    if delta <= f64::EPSILON {
        0.0
    } else if (max - red).abs() <= f64::EPSILON {
        60.0 * ((green - blue) / delta).rem_euclid(6.0)
    } else if (max - green).abs() <= f64::EPSILON {
        60.0 * (((blue - red) / delta) + 2.0)
    } else {
        60.0 * (((red - green) / delta) + 4.0)
    }
}

fn rgba_to_hsva(color: ColorValue) -> [f64; 4] {
    let red = f64::from(color.red);
    let green = f64::from(color.green);
    let blue = f64::from(color.blue);
    let max = red.max(green).max(blue);
    let min = red.min(green).min(blue);
    let delta = max - min;
    let saturation = if max <= f64::EPSILON { 0.0 } else { delta / max };
    [
        color_hue(red, green, blue, max, delta),
        saturation,
        max,
        f64::from(color.alpha),
    ]
}

fn rgba_to_hsla(color: ColorValue) -> [f64; 4] {
    let red = f64::from(color.red);
    let green = f64::from(color.green);
    let blue = f64::from(color.blue);
    let max = red.max(green).max(blue);
    let min = red.min(green).min(blue);
    let delta = max - min;
    let lightness = (max + min) * 0.5;
    let saturation = if delta <= f64::EPSILON {
        0.0
    } else {
        delta / (1.0 - (2.0 * lightness - 1.0).abs())
    };
    [
        color_hue(red, green, blue, max, delta),
        saturation,
        lightness,
        f64::from(color.alpha),
    ]
}

fn rgba_to_cmyk(color: ColorValue) -> [f64; 4] {
    let red = f64::from(color.red);
    let green = f64::from(color.green);
    let blue = f64::from(color.blue);
    let key = 1.0 - red.max(green).max(blue);
    if key >= 1.0 - f64::EPSILON {
        return [0.0, 0.0, 0.0, 1.0];
    }
    let denominator = 1.0 - key;
    [
        (1.0 - red - key) / denominator,
        (1.0 - green - key) / denominator,
        (1.0 - blue - key) / denominator,
        key,
    ]
}

fn numeric_binary(left: &RuntimeValue, right: &RuntimeValue, operator: MathOperator) -> Result<RuntimeValue, String> {
    match (left, right) {
        (RuntimeValue::Int(left), RuntimeValue::Int(right)) => {
            return match operator {
                MathOperator::Add => Ok(RuntimeValue::Int(left + right)),
                MathOperator::Subtract => Ok(RuntimeValue::Int(left - right)),
                MathOperator::Multiply => Ok(RuntimeValue::Int(left * right)),
                MathOperator::Divide => {
                    if *right == 0 {
                        Err("Math divide input cannot be zero".into())
                    } else {
                        Ok(RuntimeValue::Float(*left as f64 / *right as f64))
                    }
                }
                MathOperator::Modulo => {
                    if *right == 0 {
                        Err("Math modulo input cannot be zero".into())
                    } else {
                        Ok(RuntimeValue::Int(left % right))
                    }
                }
            };
        }
        (RuntimeValue::Vec2(left), RuntimeValue::Vec2(right)) => {
            return Ok(RuntimeValue::Vec2([
                numeric_scalar(left[0], right[0], operator)?,
                numeric_scalar(left[1], right[1], operator)?,
            ]));
        }
        (RuntimeValue::Vec3(left), RuntimeValue::Vec3(right)) => {
            return Ok(RuntimeValue::Vec3([
                numeric_scalar(left[0], right[0], operator)?,
                numeric_scalar(left[1], right[1], operator)?,
                numeric_scalar(left[2], right[2], operator)?,
            ]));
        }
        (RuntimeValue::Color(left), RuntimeValue::Color(right)) => {
            return Ok(RuntimeValue::Color(ColorValue {
                red: numeric_scalar(f64::from(left.red), f64::from(right.red), operator)? as f32,
                green: numeric_scalar(f64::from(left.green), f64::from(right.green), operator)? as f32,
                blue: numeric_scalar(f64::from(left.blue), f64::from(right.blue), operator)? as f32,
                alpha: numeric_scalar(f64::from(left.alpha), f64::from(right.alpha), operator)? as f32,
            }));
        }
        _ => {}
    }
    Ok(RuntimeValue::Float(numeric_scalar(
        value_to_f64(left),
        value_to_f64(right),
        operator,
    )?))
}

fn numeric_scalar(left: f64, right: f64, operator: MathOperator) -> Result<f64, String> {
    match operator {
        MathOperator::Add => Ok(left + right),
        MathOperator::Subtract => Ok(left - right),
        MathOperator::Multiply => Ok(left * right),
        MathOperator::Divide => {
            if right.abs() <= f64::EPSILON {
                Err("Math divide input cannot be zero".into())
            } else {
                Ok(left / right)
            }
        }
        MathOperator::Modulo => {
            if right.abs() <= f64::EPSILON {
                Err("Math modulo input cannot be zero".into())
            } else {
                Ok(left % right)
            }
        }
    }
}

fn numeric_map(value: &RuntimeValue, mapper: impl Fn(f64) -> f64) -> RuntimeValue {
    numeric_map_checked(value, |value| Ok(mapper(value))).expect("infallible numeric map")
}

fn numeric_map_checked(
    value: &RuntimeValue,
    mapper: impl Fn(f64) -> Result<f64, String>,
) -> Result<RuntimeValue, String> {
    Ok(match value {
        RuntimeValue::Int(value) => RuntimeValue::Int(mapper(*value as f64)? as i64),
        RuntimeValue::Float(value) => RuntimeValue::Float(mapper(*value)?),
        RuntimeValue::Vec2(value) => RuntimeValue::Vec2([mapper(value[0])?, mapper(value[1])?]),
        RuntimeValue::Vec3(value) => RuntimeValue::Vec3([mapper(value[0])?, mapper(value[1])?, mapper(value[2])?]),
        RuntimeValue::Color(value) => RuntimeValue::Color(ColorValue {
            red: mapper(f64::from(value.red))? as f32,
            green: mapper(f64::from(value.green))? as f32,
            blue: mapper(f64::from(value.blue))? as f32,
            alpha: mapper(f64::from(value.alpha))? as f32,
        }),
        _ => RuntimeValue::Float(mapper(value_to_f64(value))?),
    })
}

fn value_to_f64(value: &RuntimeValue) -> f64 {
    match value {
        RuntimeValue::Unit => 0.0,
        RuntimeValue::Bool(value) => f64::from(*value),
        RuntimeValue::Trigger(value) => f64::from(value.fired),
        RuntimeValue::Int(value) => *value as f64,
        RuntimeValue::Float(value) => *value,
        RuntimeValue::String(value) => value.trim().parse::<f64>().unwrap_or(0.0),
        RuntimeValue::Vec2(value) => value[0],
        RuntimeValue::Vec3(value) => value[0],
        RuntimeValue::Color(value) => f64::from(value.red),
        RuntimeValue::Duration(value) => value.as_secs_f64(),
        RuntimeValue::Array(value) => value.first().map_or(0.0, value_to_f64),
        RuntimeValue::Ref(_) | RuntimeValue::Extension(_) => 0.0,
    }
}

fn value_to_i64(value: &RuntimeValue) -> i64 {
    let value = value_to_f64(value);
    if value.is_finite() { value as i64 } else { 0 }
}

fn decimal_string(value: &RuntimeValue, decimals: usize) -> String {
    match value {
        RuntimeValue::Int(value) => value.to_string(),
        RuntimeValue::Float(value) => format!("{value:.decimals$}"),
        _ => format_runtime_value(value, decimals),
    }
}

fn format_runtime_value(value: &RuntimeValue, decimals: usize) -> String {
    match value {
        RuntimeValue::Unit => String::new(),
        RuntimeValue::Bool(value) => value.to_string(),
        RuntimeValue::Trigger(value) => value.fired.to_string(),
        RuntimeValue::Int(value) => value.to_string(),
        RuntimeValue::Float(value) => format!("{value:.decimals$}"),
        RuntimeValue::String(value) => value.to_string(),
        RuntimeValue::Vec2(value) => format!(
            "[{},{}]",
            format_float(value[0], decimals),
            format_float(value[1], decimals)
        ),
        RuntimeValue::Vec3(value) => format!(
            "[{},{},{}]",
            format_float(value[0], decimals),
            format_float(value[1], decimals),
            format_float(value[2], decimals)
        ),
        RuntimeValue::Color(value) => format!(
            "[{},{},{},{}]",
            format_float(f64::from(value.red), decimals),
            format_float(f64::from(value.green), decimals),
            format_float(f64::from(value.blue), decimals),
            format_float(f64::from(value.alpha), decimals)
        ),
        RuntimeValue::Duration(value) => time_string(value.as_secs_f64(), decimals),
        RuntimeValue::Array(values) => values
            .iter()
            .map(|value| format_runtime_value(value, decimals))
            .collect::<Vec<_>>()
            .join(","),
        RuntimeValue::Ref(value) => value.stable_id.to_string(),
        RuntimeValue::Extension(value) => value.payload.iter().map(|byte| format!("{byte:02x}")).collect(),
    }
}

fn format_float(value: f64, decimals: usize) -> String {
    format!("{value:.decimals$}")
}

fn time_string(seconds: f64, decimals: usize) -> String {
    let safe_seconds = seconds.max(0.0);
    let hours = (safe_seconds / 3600.0).floor() as u64;
    let minutes = ((safe_seconds % 3600.0) / 60.0).floor() as u64;
    let seconds = safe_seconds % 60.0;
    if decimals == 0 {
        format!("{hours:02}:{minutes:02}:{:02}", seconds.round() as u64)
    } else {
        let width = 3 + decimals;
        format!("{hours:02}:{minutes:02}:{seconds:0width$.decimals$}")
    }
}

fn brightness(value: &RuntimeValue) -> f64 {
    match value {
        RuntimeValue::Color(value) => {
            0.2126 * f64::from(value.red) + 0.7152 * f64::from(value.green) + 0.0722 * f64::from(value.blue)
        }
        _ => value_to_f64(value).abs(),
    }
}

fn delta_seconds(duration: Duration) -> f64 {
    duration.as_secs_f64().max(1.0 / 120.0)
}

fn smoothing_alpha(cutoff: f64, dt: f64) -> f64 {
    let tau = 1.0 / (2.0 * std::f64::consts::PI * cutoff.max(0.0001));
    (dt / (tau + dt)).clamp(0.0, 1.0)
}

fn state_values(state: Option<&RuntimeValue>, minimum_len: usize) -> Vec<f64> {
    let mut values = match state {
        Some(RuntimeValue::Array(values)) => values.iter().map(value_to_f64).collect(),
        Some(RuntimeValue::Vec2(value)) => value.to_vec(),
        Some(RuntimeValue::Vec3(value)) => value.to_vec(),
        Some(RuntimeValue::Float(value)) => vec![*value],
        Some(RuntimeValue::Int(value)) => vec![*value as f64],
        _ => Vec::new(),
    };
    values.resize(minimum_len, 0.0);
    values
}

fn set_state_values(state: Option<&mut RuntimeValue>, values: Vec<f64>) {
    if let Some(state) = state {
        *state = RuntimeValue::Array(values.into_iter().map(RuntimeValue::Float).collect());
    }
}

fn trim_history(history: &mut Vec<f64>, window: usize) {
    if history.len() > window {
        let remove_count = history.len() - window;
        history.drain(0..remove_count);
    }
}

fn trigger(fired: bool, edge_id: u64, logical_tick: u64) -> TriggerValue {
    if fired {
        TriggerValue::fired(edge_id, logical_tick)
    } else {
        TriggerValue::default()
    }
}
