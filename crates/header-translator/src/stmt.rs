use std::borrow::Cow;
use std::collections::HashSet;
use std::fmt;
use std::iter;

use clang::{Entity, EntityKind, EntityVisitResult};

use crate::availability::Availability;
use crate::config::{ClassData, Config};
use crate::expr::Expr;
use crate::method::{handle_reserved, Method};
use crate::rust_type::{GenericType, Ty};
use crate::unexposed_macro::UnexposedMacro;

#[derive(serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Derives(Cow<'static, str>);

impl Default for Derives {
    fn default() -> Self {
        Derives("Debug, PartialEq, Eq, Hash".into())
    }
}

impl fmt::Display for Derives {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#[derive({})]", self.0)
    }
}

/// Takes one of:
/// - `EntityKind::ObjCInterfaceDecl`
/// - `EntityKind::ObjCProtocolDecl`
/// - `EntityKind::ObjCCategoryDecl`
fn parse_objc_decl(
    entity: &Entity<'_>,
    mut superclass: Option<&mut Option<Option<GenericType>>>,
    mut generics: Option<&mut Vec<GenericType>>,
    data: Option<&ClassData>,
) -> (Vec<String>, Vec<Method>) {
    let mut protocols = Vec::new();
    let mut methods = Vec::new();

    // Track seen properties, so that when methods are autogenerated by the
    // compiler from them, we can skip them
    let mut properties = HashSet::new();

    entity.visit_children(|entity, _parent| {
        match entity.get_kind() {
            EntityKind::ObjCExplicitProtocolImpl if generics.is_none() && superclass.is_none() => {
                // TODO NS_PROTOCOL_REQUIRES_EXPLICIT_IMPLEMENTATION
            }
            EntityKind::ObjCIvarDecl if superclass.is_some() => {
                // Explicitly ignored
            }
            EntityKind::ObjCSuperClassRef => {
                if let Some(superclass) = &mut superclass {
                    let name = entity.get_name().expect("superclass name");
                    **superclass = Some(Some(GenericType {
                        name,
                        // These are filled out in EntityKind::TypeRef
                        generics: Vec::new(),
                    }));
                } else {
                    panic!("unsupported superclass {entity:?}");
                }
            }
            EntityKind::ObjCRootClass => {
                if let Some(superclass) = &mut superclass {
                    // TODO: Maybe just skip root classes entirely?
                    **superclass = Some(None);
                } else {
                    panic!("unsupported root class {entity:?}");
                }
            }
            EntityKind::ObjCClassRef if generics.is_some() => {
                // println!("ObjCClassRef: {:?}", entity.get_display_name());
            }
            EntityKind::TemplateTypeParameter => {
                if let Some(generics) = &mut generics {
                    // TODO: Generics with bounds (like NSMeasurement<UnitType: NSUnit *>)
                    // let ty = entity.get_type().expect("template type");
                    let name = entity.get_display_name().expect("template name");
                    generics.push(GenericType {
                        name,
                        generics: Vec::new(),
                    });
                } else {
                    panic!("unsupported generics {entity:?}");
                }
            }
            EntityKind::ObjCProtocolRef => {
                protocols.push(entity.get_name().expect("protocolref to have name"));
            }
            EntityKind::ObjCInstanceMethodDecl | EntityKind::ObjCClassMethodDecl => {
                let partial = Method::partial(entity);

                if !properties.remove(&(partial.is_class, partial.fn_name.clone())) {
                    let data = data
                        .map(|data| {
                            data.methods
                                .get(&partial.fn_name)
                                .copied()
                                .unwrap_or_default()
                        })
                        .unwrap_or_default();
                    if let Some(method) = partial.parse(data) {
                        methods.push(method);
                    }
                }
            }
            EntityKind::ObjCPropertyDecl => {
                let partial = Method::partial_property(entity);

                assert!(
                    properties.insert((partial.is_class, partial.getter_name.clone())),
                    "already exisiting property"
                );
                if let Some(setter_name) = partial.setter_name.clone() {
                    assert!(
                        properties.insert((partial.is_class, setter_name)),
                        "already exisiting property"
                    );
                }

                let getter_data = data
                    .map(|data| {
                        data.methods
                            .get(&partial.getter_name)
                            .copied()
                            .unwrap_or_default()
                    })
                    .unwrap_or_default();
                let setter_data = partial.setter_name.as_ref().map(|setter_name| {
                    data.map(|data| data.methods.get(setter_name).copied().unwrap_or_default())
                        .unwrap_or_default()
                });

                let (getter, setter) = partial.parse(getter_data, setter_data);
                if let Some(getter) = getter {
                    methods.push(getter);
                }
                if let Some(setter) = setter {
                    methods.push(setter);
                }
            }
            EntityKind::VisibilityAttr => {
                // Already exposed as entity.get_visibility()
            }
            EntityKind::TypeRef => {
                let name = entity.get_name().expect("typeref name");
                if let Some(Some(Some(GenericType { generics, .. }))) = &mut superclass {
                    generics.push(GenericType {
                        name,
                        generics: Vec::new(),
                    });
                } else {
                    panic!("unsupported typeref {entity:?}");
                }
            }
            EntityKind::ObjCException if superclass.is_some() => {
                // Maybe useful for knowing when to implement `Error` for the type
            }
            EntityKind::UnexposedAttr => {
                if let Some(macro_) = UnexposedMacro::parse(&entity) {
                    println!("objc decl {entity:?}: {macro_:?}");
                }
            }
            _ => panic!("unknown objc decl child {entity:?}"),
        };
        EntityVisitResult::Continue
    });

    if !properties.is_empty() {
        if properties == HashSet::from([(false, "setDisplayName".to_owned())]) {
            // TODO
        } else {
            panic!("did not properly add methods to properties:\n{methods:?}\n{properties:?}");
        }
    }

    (protocols, methods)
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// @interface name: superclass <protocols*>
    /// ->
    /// extern_class!
    ClassDecl {
        ty: GenericType,
        availability: Availability,
        superclass: Option<GenericType>,
        derives: Derives,
    },
    /// @interface class_name (name) <protocols*>
    /// ->
    /// extern_methods!
    Methods {
        ty: GenericType,
        availability: Availability,
        methods: Vec<Method>,
        /// For the categories that have a name (though some don't, see NSClipView)
        category_name: Option<String>,
    },
    /// @protocol name <protocols*>
    /// ->
    /// extern_protocol!
    ProtocolDecl {
        name: String,
        availability: Availability,
        protocols: Vec<String>,
        methods: Vec<Method>,
    },
    /// @interface ty: _ <protocols*>
    /// @interface ty (_) <protocols*>
    ProtocolImpl {
        ty: GenericType,
        availability: Availability,
        protocol: String,
    },
    /// struct name {
    ///     fields*
    /// };
    ///
    /// typedef struct {
    ///     fields*
    /// } name;
    ///
    /// typedef struct _name {
    ///     fields*
    /// } name;
    StructDecl {
        name: String,
        boxable: bool,
        fields: Vec<(String, Ty)>,
    },
    /// typedef NS_OPTIONS(type, name) {
    ///     variants*
    /// };
    ///
    /// typedef NS_ENUM(type, name) {
    ///     variants*
    /// };
    ///
    /// enum name {
    ///     variants*
    /// };
    ///
    /// enum {
    ///     variants*
    /// };
    EnumDecl {
        name: Option<String>,
        ty: Ty,
        kind: Option<UnexposedMacro>,
        variants: Vec<(String, Expr)>,
    },
    /// static const ty name = expr;
    /// extern const ty name;
    VarDecl {
        name: String,
        ty: Ty,
        value: Option<Expr>,
    },
    /// extern ret name(args*);
    ///
    /// static inline ret name(args*) {
    ///     body
    /// }
    FnDecl {
        name: String,
        arguments: Vec<(String, Ty)>,
        result_type: Ty,
        // Some -> inline function.
        body: Option<()>,
    },
    /// typedef Type TypedefName;
    AliasDecl { name: String, ty: Ty },
}

fn parse_struct(entity: &Entity<'_>, name: String) -> Stmt {
    let mut boxable = false;
    let mut fields = Vec::new();

    entity.visit_children(|entity, _parent| {
        match entity.get_kind() {
            EntityKind::UnexposedAttr => {
                if let Some(macro_) = UnexposedMacro::parse(&entity) {
                    panic!("unexpected attribute: {macro_:?}");
                }
            }
            EntityKind::FieldDecl => {
                let name = entity.get_name().expect("struct field name");
                let ty = entity.get_type().expect("struct field type");
                let ty = Ty::parse_struct_field(ty);

                if entity.is_bit_field() {
                    println!("[UNSOUND] struct bitfield {name}: {entity:?}");
                }

                fields.push((name, ty))
            }
            EntityKind::ObjCBoxable => {
                boxable = true;
            }
            _ => panic!("unknown struct field {entity:?}"),
        }
        EntityVisitResult::Continue
    });

    Stmt::StructDecl {
        name,
        boxable,
        fields,
    }
}

impl Stmt {
    pub fn parse(entity: &Entity<'_>, config: &Config) -> Vec<Self> {
        match entity.get_kind() {
            // These are inconsequential for us, since we resolve imports differently
            EntityKind::ObjCClassRef | EntityKind::ObjCProtocolRef => vec![],
            EntityKind::ObjCInterfaceDecl => {
                // entity.get_mangled_objc_names()
                let name = entity.get_name().expect("class name");
                let class_data = config.class_data.get(&name);

                if class_data.map(|data| data.skipped).unwrap_or_default() {
                    return vec![];
                }

                let availability = Availability::parse(
                    entity
                        .get_platform_availability()
                        .expect("class availability"),
                );
                // println!("Availability: {:?}", entity.get_platform_availability());
                let mut superclass = None;
                let mut generics = Vec::new();

                let (protocols, methods) = parse_objc_decl(
                    &entity,
                    Some(&mut superclass),
                    Some(&mut generics),
                    class_data,
                );

                let ty = GenericType { name, generics };

                (!class_data
                    .map(|data| data.definition_skipped)
                    .unwrap_or_default())
                .then(|| Self::ClassDecl {
                    ty: ty.clone(),
                    availability: availability.clone(),
                    superclass: superclass.expect("no superclass found"),
                    derives: class_data
                        .map(|data| data.derives.clone())
                        .unwrap_or_default(),
                })
                .into_iter()
                .chain(protocols.into_iter().map(|protocol| Self::ProtocolImpl {
                    ty: ty.clone(),
                    availability: availability.clone(),
                    protocol,
                }))
                .chain(iter::once(Self::Methods {
                    ty: ty.clone(),
                    availability: availability.clone(),
                    methods,
                    category_name: None,
                }))
                .collect()
            }
            EntityKind::ObjCCategoryDecl => {
                let name = entity.get_name();
                let availability = Availability::parse(
                    entity
                        .get_platform_availability()
                        .expect("category availability"),
                );

                let mut class_name = None;
                entity.visit_children(|entity, _parent| {
                    if entity.get_kind() == EntityKind::ObjCClassRef {
                        if class_name.is_some() {
                            panic!("could not find unique category class")
                        }
                        class_name = Some(entity.get_name().expect("class name"));
                        EntityVisitResult::Break
                    } else {
                        EntityVisitResult::Continue
                    }
                });
                let class_name = class_name.expect("could not find category class");
                let class_data = config.class_data.get(&class_name);

                if class_data.map(|data| data.skipped).unwrap_or_default() {
                    return vec![];
                }

                let mut class_generics = Vec::new();

                let (protocols, methods) =
                    parse_objc_decl(&entity, None, Some(&mut class_generics), class_data);

                let ty = GenericType {
                    name: class_name,
                    generics: class_generics,
                };

                iter::once(Self::Methods {
                    ty: ty.clone(),
                    availability: availability.clone(),
                    methods,
                    category_name: name,
                })
                .into_iter()
                .chain(protocols.into_iter().map(|protocol| Self::ProtocolImpl {
                    ty: ty.clone(),
                    availability: availability.clone(),
                    protocol,
                }))
                .collect()
            }
            EntityKind::ObjCProtocolDecl => {
                let name = entity.get_name().expect("protocol name");
                let protocol_data = config.protocol_data.get(&name);

                if protocol_data.map(|data| data.skipped).unwrap_or_default() {
                    return vec![];
                }

                let availability = Availability::parse(
                    entity
                        .get_platform_availability()
                        .expect("protocol availability"),
                );

                let (protocols, methods) = parse_objc_decl(&entity, None, None, protocol_data);

                vec![Self::ProtocolDecl {
                    name,
                    availability,
                    protocols,
                    methods,
                }]
            }
            EntityKind::TypedefDecl => {
                let name = entity.get_name().expect("typedef name");
                let mut struct_ = None;
                let mut skip_struct = false;

                entity.visit_children(|entity, _parent| {
                    match entity.get_kind() {
                        // TODO: Parse NS_TYPED_EXTENSIBLE_ENUM vs. NS_TYPED_ENUM
                        EntityKind::UnexposedAttr => {
                            if let Some(macro_) = UnexposedMacro::parse(&entity) {
                                panic!("unexpected attribute: {macro_:?}");
                            }
                        }
                        EntityKind::StructDecl => {
                            if config
                                .struct_data
                                .get(&name)
                                .map(|data| data.skipped)
                                .unwrap_or_default()
                            {
                                skip_struct = true;
                                return EntityVisitResult::Continue;
                            }

                            let struct_name = entity.get_name();
                            if struct_name
                                .map(|name| name.starts_with('_'))
                                .unwrap_or(true)
                            {
                                // If this struct doesn't have a name, or the
                                // name is private, let's parse it with the
                                // typedef name.
                                struct_ = Some(parse_struct(&entity, name.clone()))
                            } else {
                                skip_struct = true;
                            }
                        }
                        EntityKind::ObjCClassRef
                        | EntityKind::ObjCProtocolRef
                        | EntityKind::TypeRef
                        | EntityKind::ParmDecl => {}
                        _ => panic!("unknown typedef child in {name}: {entity:?}"),
                    };
                    EntityVisitResult::Continue
                });

                if let Some(struct_) = struct_ {
                    return vec![struct_];
                }

                if skip_struct {
                    return vec![];
                }

                if config
                    .typedef_data
                    .get(&name)
                    .map(|data| data.skipped)
                    .unwrap_or_default()
                {
                    return vec![];
                }

                let ty = entity
                    .get_typedef_underlying_type()
                    .expect("typedef underlying type");
                if let Some(ty) = Ty::parse_typedef(ty) {
                    vec![Self::AliasDecl { name, ty }]
                } else {
                    vec![]
                }
            }
            EntityKind::StructDecl => {
                if let Some(name) = entity.get_name() {
                    if config
                        .struct_data
                        .get(&name)
                        .map(|data| data.skipped)
                        .unwrap_or_default()
                    {
                        return vec![];
                    }
                    if !name.starts_with('_') {
                        return vec![parse_struct(entity, name)];
                    }
                }
                vec![]
            }
            EntityKind::EnumDecl => {
                // Enum declarations show up twice for some reason, but
                // luckily this flag is set on the least descriptive entity.
                if !entity.is_definition() {
                    return vec![];
                }

                let name = entity.get_name();

                let data = config
                    .enum_data
                    .get(name.as_deref().unwrap_or("anonymous"))
                    .cloned()
                    .unwrap_or_default();
                if data.skipped {
                    return vec![];
                }

                let ty = entity.get_enum_underlying_type().expect("enum type");
                let is_signed = ty.is_signed_integer();
                let ty = Ty::parse_enum(ty);
                let mut kind = None;
                let mut variants = Vec::new();

                entity.visit_children(|entity, _parent| {
                    match entity.get_kind() {
                        EntityKind::EnumConstantDecl => {
                            let name = entity.get_name().expect("enum constant name");

                            if data
                                .constants
                                .get(&name)
                                .map(|data| data.skipped)
                                .unwrap_or_default()
                            {
                                return EntityVisitResult::Continue;
                            }

                            let val = Expr::from_val(
                                entity
                                    .get_enum_constant_value()
                                    .expect("enum constant value"),
                                is_signed,
                            );
                            let expr = if data.use_value {
                                val
                            } else {
                                Expr::parse_enum_constant(&entity).unwrap_or(val)
                            };
                            variants.push((name, expr));
                        }
                        EntityKind::UnexposedAttr => {
                            if let Some(macro_) = UnexposedMacro::parse(&entity) {
                                if let Some(kind) = &kind {
                                    assert_eq!(
                                        kind, &macro_,
                                        "got differing enum kinds in {name:?}"
                                    );
                                } else {
                                    kind = Some(macro_);
                                }
                            }
                        }
                        EntityKind::FlagEnum => {
                            let macro_ = UnexposedMacro::Options;
                            if let Some(kind) = &kind {
                                assert_eq!(kind, &macro_, "got differing enum kinds in {name:?}");
                            } else {
                                kind = Some(macro_);
                            }
                        }
                        _ => {
                            panic!("unknown enum child {entity:?} in {name:?}");
                        }
                    }
                    EntityVisitResult::Continue
                });

                vec![Self::EnumDecl {
                    name,
                    ty,
                    kind,
                    variants,
                }]
            }
            EntityKind::VarDecl => {
                let name = entity.get_name().expect("var decl name");

                if config
                    .statics
                    .get(&name)
                    .map(|data| data.skipped)
                    .unwrap_or_default()
                {
                    return vec![];
                }

                let ty = entity.get_type().expect("var type");
                let ty = Ty::parse_static(ty);
                let mut value = None;

                entity.visit_children(|entity, _parent| {
                    match entity.get_kind() {
                        EntityKind::UnexposedAttr => {
                            if let Some(macro_) = UnexposedMacro::parse(&entity) {
                                panic!("unexpected attribute: {macro_:?}");
                            }
                        }
                        EntityKind::VisibilityAttr => {}
                        EntityKind::ObjCClassRef => {}
                        EntityKind::TypeRef => {}
                        _ if entity.is_expression() => {
                            if value.is_none() {
                                value = Some(Expr::parse_var(&entity));
                            } else {
                                panic!("got variable value twice")
                            }
                        }
                        _ => panic!("unknown vardecl child in {name}: {entity:?}"),
                    };
                    EntityVisitResult::Continue
                });

                let value = match value {
                    Some(Some(expr)) => Some(expr),
                    Some(None) => {
                        println!("skipped static {name}");
                        return vec![];
                    }
                    None => None,
                };

                vec![Self::VarDecl { name, ty, value }]
            }
            EntityKind::FunctionDecl => {
                let name = entity.get_name().expect("function name");

                if config
                    .fns
                    .get(&name)
                    .map(|data| data.skipped)
                    .unwrap_or_default()
                {
                    return vec![];
                }

                if entity.is_variadic() {
                    println!("can't handle variadic function {name}");
                    return vec![];
                }

                let result_type = entity.get_result_type().expect("function result type");
                let result_type = Ty::parse_function_return(result_type);
                let mut arguments = Vec::new();

                assert!(
                    !entity.is_static_method(),
                    "unexpected static method {name}"
                );

                entity.visit_children(|entity, _parent| {
                    match entity.get_kind() {
                        EntityKind::UnexposedAttr => {
                            if let Some(macro_) = UnexposedMacro::parse(&entity) {
                                panic!("unexpected function attribute: {macro_:?}");
                            }
                        }
                        EntityKind::ObjCClassRef | EntityKind::TypeRef => {}
                        EntityKind::ParmDecl => {
                            // Could also be retrieved via. `get_arguments`
                            let name = entity.get_name().unwrap_or_else(|| "_".into());
                            let ty = entity.get_type().expect("function argument type");
                            let ty = Ty::parse_function_argument(ty);
                            arguments.push((name, ty))
                        }
                        _ => panic!("unknown function child in {name}: {entity:?}"),
                    };
                    EntityVisitResult::Continue
                });

                let body = if entity.is_inline_function() {
                    Some(())
                } else {
                    None
                };

                vec![Self::FnDecl {
                    name,
                    arguments,
                    result_type,
                    body,
                }]
            }
            EntityKind::UnionDecl => {
                // println!(
                //     "union: {:?}, {:?}, {:#?}, {:#?}",
                //     entity.get_display_name(),
                //     entity.get_name(),
                //     entity.has_attributes(),
                //     entity.get_children(),
                // );
                vec![]
            }
            _ => {
                panic!("Unknown: {:?}", entity)
            }
        }
    }

    pub fn compare(&self, other: &Self) {
        if self != other {
            match (&self, &other) {
                (
                    Self::Methods {
                        methods: self_methods,
                        ..
                    },
                    Self::Methods {
                        methods: other_methods,
                        ..
                    },
                ) => {
                    super::compare_vec(
                        &self_methods,
                        &other_methods,
                        |_i, self_method, other_method| {
                            assert_eq!(self_method, other_method, "methods were not equal");
                        },
                    );
                }
                _ => {}
            }

            panic!("statements were not equal:\n{self:#?}\n{other:#?}");
        }
    }
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct GenericTyHelper<'a>(&'a GenericType);

        impl fmt::Display for GenericTyHelper<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0.name)?;
                if !self.0.generics.is_empty() {
                    write!(f, "<")?;
                    for generic in &self.0.generics {
                        write!(f, "{generic}, ")?;
                    }
                    for generic in &self.0.generics {
                        write!(f, "{generic}Ownership, ")?;
                    }
                    write!(f, ">")?;
                }
                Ok(())
            }
        }

        struct GenericParamsHelper<'a>(&'a [GenericType]);

        impl fmt::Display for GenericParamsHelper<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                if !self.0.is_empty() {
                    write!(f, "<")?;
                    for generic in self.0 {
                        write!(f, "{generic}: Message, ")?;
                    }
                    for generic in self.0 {
                        write!(f, "{generic}Ownership: Ownership, ")?;
                    }
                    write!(f, ">")?;
                }
                Ok(())
            }
        }

        match self {
            Self::ClassDecl {
                ty,
                availability: _,
                superclass,
                derives,
            } => {
                let default_superclass = GenericType {
                    name: "Object".into(),
                    generics: Vec::new(),
                };
                let superclass = superclass.as_ref().unwrap_or_else(|| &default_superclass);

                // TODO: Use ty.get_objc_protocol_declarations()

                let macro_name = if ty.generics.is_empty() {
                    "extern_class"
                } else {
                    "__inner_extern_class"
                };

                writeln!(f, "{macro_name}!(")?;
                writeln!(f, "    {}", derives)?;
                write!(f, "    pub struct ")?;
                if ty.generics.is_empty() {
                    write!(f, "{}", ty.name)?;
                } else {
                    write!(f, "{}<", ty.name)?;
                    for generic in &ty.generics {
                        write!(f, "{generic}: Message = Object, ")?;
                    }
                    for generic in &ty.generics {
                        write!(f, "{generic}Ownership: Ownership = Shared, ")?;
                    }
                    write!(f, ">")?;
                };
                if ty.generics.is_empty() {
                    writeln!(f, ";")?;
                } else {
                    writeln!(f, " {{")?;
                    for (i, generic) in ty.generics.iter().enumerate() {
                        // Invariant over the generic (for now)
                        writeln!(
                            f,
                            "_inner{i}: PhantomData<*mut ({generic}, {generic}Ownership)>,"
                        )?;
                    }
                    writeln!(f, "notunwindsafe: PhantomData<&'static mut ()>,")?;
                    writeln!(f, "}}")?;
                }
                writeln!(f, "")?;
                writeln!(
                    f,
                    "    unsafe impl{} ClassType for {} {{",
                    GenericParamsHelper(&ty.generics),
                    GenericTyHelper(&ty)
                )?;
                writeln!(f, "        type Super = {};", GenericTyHelper(&superclass))?;
                writeln!(f, "    }}")?;
                writeln!(f, ");")?;
            }
            Self::Methods {
                ty,
                availability: _,
                methods,
                category_name,
            } => {
                writeln!(f, "extern_methods!(")?;
                if let Some(category_name) = category_name {
                    writeln!(f, "    /// {category_name}")?;
                }
                writeln!(
                    f,
                    "    unsafe impl{} {} {{",
                    GenericParamsHelper(&ty.generics),
                    GenericTyHelper(&ty)
                )?;
                for method in methods {
                    writeln!(f, "{method}")?;
                }
                writeln!(f, "    }}")?;
                writeln!(f, ");")?;
            }
            Self::ProtocolImpl {
                ty: _,
                availability: _,
                protocol: _,
            } => {
                // TODO
            }
            Self::ProtocolDecl {
                name,
                availability: _,
                protocols: _,
                methods,
            } => {
                writeln!(f, "extern_protocol!(")?;
                writeln!(f, "    pub struct {name};")?;
                writeln!(f, "")?;
                writeln!(f, "    unsafe impl ProtocolType for {name} {{")?;
                for method in methods {
                    writeln!(f, "{method}")?;
                }
                writeln!(f, "    }}")?;
                writeln!(f, ");")?;
            }
            Self::StructDecl {
                name,
                boxable: _,
                fields,
            } => {
                writeln!(f, "extern_struct!(")?;
                writeln!(f, "    pub struct {name} {{")?;
                for (name, ty) in fields {
                    write!(f, "        ")?;
                    if !name.starts_with('_') {
                        write!(f, "pub ")?;
                    }
                    writeln!(f, "{name}: {ty},")?;
                }
                writeln!(f, "    }}")?;
                writeln!(f, ");")?;
            }
            Self::EnumDecl {
                name,
                ty,
                kind,
                variants,
            } => {
                let macro_name = match kind {
                    None => "extern_enum",
                    Some(UnexposedMacro::Enum) => "ns_enum",
                    Some(UnexposedMacro::Options) => "ns_options",
                    Some(UnexposedMacro::ClosedEnum) => "ns_closed_enum",
                    Some(UnexposedMacro::ErrorEnum) => "ns_error_enum",
                };
                writeln!(f, "{}!(", macro_name)?;
                writeln!(f, "    #[underlying({ty})]")?;
                write!(f, "    pub enum ",)?;
                if let Some(name) = name {
                    write!(f, "{name} ")?;
                }
                writeln!(f, "{{")?;
                for (name, expr) in variants {
                    writeln!(f, "        {name} = {expr},")?;
                }
                writeln!(f, "    }}")?;
                writeln!(f, ");")?;
            }
            Self::VarDecl {
                name,
                ty,
                value: None,
            } => {
                writeln!(f, "extern_static!({name}: {ty});")?;
            }
            Self::VarDecl {
                name,
                ty,
                value: Some(expr),
            } => {
                writeln!(f, "extern_static!({name}: {ty} = {expr});")?;
            }
            Self::FnDecl {
                name,
                arguments,
                result_type,
                body: None,
            } => {
                writeln!(f, "extern_fn!(")?;
                write!(f, "    pub unsafe fn {name}(")?;
                for (param, arg_ty) in arguments {
                    write!(f, "{}: {arg_ty},", handle_reserved(&param))?;
                }
                writeln!(f, "){result_type};")?;
                writeln!(f, ");")?;
            }
            Self::FnDecl {
                name,
                arguments,
                result_type,
                body: Some(_body),
            } => {
                writeln!(f, "inline_fn!(")?;
                write!(f, "    pub unsafe fn {name}(")?;
                for (param, arg_ty) in arguments {
                    write!(f, "{}: {arg_ty},", handle_reserved(&param))?;
                }
                writeln!(f, "){result_type} {{")?;
                writeln!(f, "        todo!()")?;
                writeln!(f, "    }}")?;
                writeln!(f, ");")?;
            }
            Self::AliasDecl { name, ty } => {
                writeln!(f, "pub type {name} = {ty};")?;
            }
        };
        Ok(())
    }
}
