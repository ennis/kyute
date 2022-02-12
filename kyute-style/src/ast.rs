use crate::{color::Color, grammar, Arena, ParseError, ParseState};
use serde::Serialize;
use serde_json as json;
use serde_json::{json, Value};
use std::{collections::HashMap, io};
use thiserror::Error;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub enum Unit {
    Degrees,
    Px,
    In,
    Dip,
    Em,
    Percentage,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub enum BorderPosition {
    Inside,
    Center,
    Outside,
}

impl Default for BorderPosition {
    fn default() -> Self {
        BorderPosition::Inside
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize)]
pub struct Dimension {
    pub value: f64,
    pub unit: Unit,
}

impl Eval for Dimension {
    fn eval(&self, ctx: &EvalCtx) -> Value {
        json::to_value(*self).unwrap()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Document<'ast> {
    pub items: &'ast [Item<'ast>],
}

pub type SymbolTable<'ast> = HashMap<&'ast str, &'ast Item<'ast>>;

pub struct EvalCtx<'ast> {
    /// Top level symbol table.
    symtab: SymbolTable<'ast>,
}

impl<'ast> EvalCtx<'ast> {
    ///
    pub fn resolve(&self, var: &str) -> Option<&'ast Item<'ast>> {
        match self.symtab.get(var) {
            None => {
                eprintln!("variable not found: {}", var);
                None
            }
            Some(Item {
                value: Expr::Var(var),
                ..
            }) => self.resolve(var),
            Some(item) => Some(item),
        }
    }
}

impl<'ast> Document<'ast> {
    pub fn parse<'input>(
        input: &'input str,
        arena: &'ast Arena,
    ) -> Result<Document<'ast>, ParseError<'input>> {
        let mut state = ParseState::new(arena);
        grammar::DocumentParser::new().parse(&mut state, input)
    }

    pub fn to_json(&self) -> json::Value {
        let mut symtab = HashMap::new();
        for item in self.items {
            symtab.insert(item.name, item);
        }

        let mut ctx = EvalCtx { symtab };

        let mut map = json::Map::new();
        for item in self.items {
            let item_json = item.value.eval(&ctx);
            map.insert(item.name.to_string(), item_json);
        }

        json::Value::Object(map)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Item<'ast> {
    pub name: &'ast str,
    pub value: Expr<'ast>,
}

impl<'ast> Item<'ast> {}

#[derive(Copy, Clone, Debug)]
pub enum Expr<'ast> {
    Color(&'ast ColorExpr<'ast>),
    Border(&'ast BorderExpr<'ast>),
    LinearGradient(&'ast LinearGradientExpr<'ast>),
    Var(&'ast str),
    Record(&'ast RecordExpr<'ast>),
}

impl<'ast> Eval for Expr<'ast> {
    fn eval(&self, ctx: &EvalCtx) -> json::Value {
        let (ty, data) = match self {
            Expr::Color(color) => (Type::Color, color.eval(ctx)),
            Expr::Border(border) => (Type::Border, border.eval(ctx)),
            Expr::LinearGradient(linear_gradient) => (Type::Paint, linear_gradient.eval(ctx)),
            Expr::Var(var) => {
                let item = ctx.resolve(var);
                if let Some(item) = item {
                    (item.value.get_type(ctx), item.value.eval(ctx))
                } else {
                    (Type::Unknown, json!({}))
                }
            }
            Expr::Record(record) => (Type::Record(record.class), record.eval(ctx)),
        };

        let ty = match ty {
            Type::Color => "color",
            Type::Border => "border",
            Type::Paint => "paint",
            Type::Record(class) => "record",
            Type::Unknown => "unknown",
            Type::Dimension => "dimension",
        };

        json!({
            "ty": ty,
            "data": data,
        })
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Type<'ast> {
    Color,
    Border,
    Paint,
    Record(Option<&'ast str>),
    Unknown,
    Dimension,
}

impl<'ast> Expr<'ast> {
    pub fn get_type(&self, ctx: &EvalCtx<'ast>) -> Type<'ast> {
        match self {
            Expr::Color(_) => Type::Color,
            Expr::Border(_) => Type::Border,
            Expr::LinearGradient(_) => Type::Paint,
            Expr::Var(var) => {
                if let Some(item) = ctx.resolve(var) {
                    item.value.get_type(ctx)
                } else {
                    Type::Unknown
                }
            }
            Expr::Record(RecordExpr { class, .. }) => Type::Record(*class),
        }
    }
}

/// Trait for AST elements that can be evaluated to a JSON value given a context.
pub trait Eval {
    fn eval(&self, ctx: &EvalCtx) -> json::Value;
}

#[derive(Copy, Clone, Debug)]
pub enum ValueOrVar<'ast, T> {
    Var(&'ast str),
    Value(T),
}

impl<'ast, T: Eval> ValueOrVar<'ast, T> {
    pub fn eval(&self, ctx: &EvalCtx<'ast>, expected_type: Type) -> json::Value {
        match self {
            ValueOrVar::Var(var) => {
                if let Some(item) = ctx.resolve(var) {
                    if item.value.get_type(ctx) != expected_type {
                        eprintln!("unexpected type");
                        json!({})
                    } else {
                        item.value.eval(ctx)
                    }
                } else {
                    json!({})
                }
            }
            ValueOrVar::Value(val) => val.eval(ctx),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ColorExpr<'ast> {
    pub color: ValueOrVar<'ast, Color>,
}

impl<'ast> Eval for ColorExpr<'ast> {
    fn eval(&self, ctx: &EvalCtx) -> json::Value {
        self.color.eval(ctx, Type::Color)
    }
}

impl Eval for Color {
    fn eval(&self, _ctx: &EvalCtx) -> json::Value {
        json!([self.red, self.green, self.blue, self.alpha])
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BorderExpr<'ast> {
    pub border: ValueOrVar<'ast, Border<'ast>>,
}

impl<'ast> Eval for BorderExpr<'ast> {
    fn eval(&self, ctx: &EvalCtx) -> Value {
        self.border.eval(ctx, Type::Border)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Border<'ast> {
    pub length: ValueOrVar<'ast, Dimension>,
    pub position: BorderPosition,
    pub color: ValueOrVar<'ast, Color>,
}

impl<'ast> Eval for Border<'ast> {
    fn eval(&self, ctx: &EvalCtx) -> Value {
        let length = self.length.eval(ctx, Type::Dimension);
        let color = self.color.eval(ctx, Type::Color);
        json!({
            "length": length,
            "position": self.position,
            "color": color,
        })
    }
}

#[derive(Copy, Clone, Debug)]
pub struct LinearGradientExpr<'ast> {
    pub orientation: Dimension,
    pub stops: &'ast [GradientStop<'ast>],
}

impl<'ast> Eval for LinearGradientExpr<'ast> {
    fn eval(&self, ctx: &EvalCtx) -> json::Value {
        let stops: Vec<_> = self.stops.iter().map(|stop| stop.eval(ctx)).collect();
        json!({
            "orientation": self.orientation,
            "stops": stops
        })
    }
}

#[derive(Copy, Clone, Debug)]
pub struct GradientStop<'ast> {
    pub color: ValueOrVar<'ast, Color>,
    pub pos1: Option<Dimension>,
    pub pos2: Option<Dimension>,
}

impl<'ast> Eval for GradientStop<'ast> {
    fn eval(&self, ctx: &EvalCtx) -> json::Value {
        let color = self.color.eval(ctx, Type::Color);
        match (self.pos1, self.pos2) {
            (None, None) => {
                json!({
                    "color": color,
                })
            }
            (Some(p1), None) => {
                json!({
                    "color": color,
                    "pos1": p1,
                })
            }
            (Some(p1), Some(p2)) => {
                json!({
                    "color": color,
                    "pos1": p1,
                    "pos2": p2,
                })
            }
            _ => {
                json!({
                    "color": color,
                })
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct RecordExpr<'ast> {
    pub class: Option<&'ast str>,
    pub attributes: &'ast [Item<'ast>],
}

impl<'ast> Eval for RecordExpr<'ast> {
    fn eval(&self, ctx: &EvalCtx) -> Value {
        let mut map = json::Map::new();
        for attrib in self.attributes {
            map.insert(attrib.name.to_string(), attrib.value.eval(ctx));
        }
        json::Value::Object(map)
    }
}

/*#[derive(Copy, Clone, Debug)]
pub struct ItemPaint<'ast> {
    pub name: &'ast str,
    pub value: Paint<'ast>,
}

#[derive(Copy, Clone, Debug)]
pub enum Paint<'ast> {
    Color(ValueOrVar<'ast, Color<'ast>>),
    LinearGradient(LinearGradient<'ast>),
}

#[derive(Copy, Clone, Debug)]
pub struct ItemConstant<'ast> {
    pub name: &'ast str,
    pub value: &'ast str,
}*/
