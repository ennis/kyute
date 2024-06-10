//! Implementation of the linear system of equations for the geometry variables in immediate mode widgets.
use crate::widgets::immediate::IMCTX;
use smallvec::{smallvec, SmallVec};
use std::{
    cell::RefCell,
    fmt,
    ops::{Add, Mul, Sub},
};
use tracing::warn;

/// Represents a variable in a linear system.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct VarId(u16);

impl VarId {
    pub fn equals(self, value: impl Into<LinExpr>) {
        IMCTX.with(|imctx| {
            // create lhs & rhs expressions before borrowing SYS
            let lhs = LinExpr::from(self);
            let rhs = value.into();
            imctx.linear_system.borrow_mut().add_equation(lhs, rhs);
        })
    }

    /// Returns the value of the variable if it is known.
    pub fn value(self) -> Option<f64> {
        IMCTX.with(|imctx| {
            let sys = imctx.linear_system.borrow();
            sys.vars[self.0 as usize].value()
        })
    }

    /// Returns the linear expression associated with a dependent variable.
    pub fn expression(self) -> Option<LinExpr> {
        IMCTX.with(|imctx| {
            let sys = imctx.linear_system.borrow();
            sys.vars[self.0 as usize].expression()
        })
    }

    /// Same as `value()` but returns 0.0 if the variable is not resolved.
    pub fn resolve(self) -> f64 {
        if let Some(value) = self.value() {
            value
        } else {
            warn!("variable is not resolved");
            0.0
        }
    }
}

pub fn var() -> VarId {
    IMCTX.with(|imctx| {
        let mut sys = imctx.linear_system.borrow_mut();
        sys.add_var()
    })
}

#[derive(Clone, Debug)]
enum Variable {
    UnknownIndependent,
    /// Value is unknown but given by a linear combination of other variables.
    Dependent(LinExpr),
}

impl Variable {
    pub fn is_independent(&self) -> bool {
        matches!(self, Variable::UnknownIndependent)
    }

    /// The linear expression associated with a variable.
    pub fn expression(&self) -> Option<LinExpr> {
        match self {
            Variable::Dependent(expr) => Some(expr.clone()),
            _ => None,
        }
    }

    pub fn is_dependent(&self) -> bool {
        matches!(self, Variable::Dependent(_))
    }

    pub fn value(&self) -> Option<f64> {
        match self {
            Variable::Dependent(expr) => expr.value(),
            _ => None,
        }
    }
    pub fn resolve(&self) -> f64 {
        match self.value() {
            Some(value) => value,
            None => {
                warn!("variable is not resolved");
                0.0
            }
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct Term {
    coef: f64,
    var: VarId,
}

/// Represents a linear expression of the form `a0 × x0 + ... + an × xn + b`.
#[derive(Clone, PartialEq)]
pub struct LinExpr {
    constant: f64,
    terms: SmallVec<Term, 4>,
}

impl LinExpr {
    /// Returns the value of the expression if it is a constant.
    fn value(&self) -> Option<f64> {
        if self.terms.is_empty() {
            Some(self.constant)
        } else {
            None
        }
    }

    /// Substitutes a variable with the given expression.
    fn subst(&mut self, var: VarId, expr: &LinExpr) {
        let mut new_expr = LinExpr {
            terms: smallvec![],
            constant: self.constant,
        };
        for term in &self.terms {
            if term.var == var {
                for expr_term in &expr.terms {
                    new_expr.add_term(term.coef * expr_term.coef, expr_term.var);
                }
                new_expr.constant += term.coef * expr.constant;
            } else {
                new_expr.add_term(term.coef, term.var);
            }
        }
        *self = new_expr;
    }

    /// Tries to evaluate the expression given the values of the variables.
    fn eval(&self, vars: &[Variable]) -> Option<f64> {
        let mut value = self.constant;
        for term in &self.terms {
            if let Some(v) = vars[term.var.0 as usize].value() {
                value += term.coef * v;
            } else {
                return None;
            }
        }
        Some(value)
    }

    fn add_term(&mut self, coef: f64, var: VarId) {
        for i in 0..self.terms.len() {
            if self.terms[i].var == var {
                self.terms[i].coef += coef;
                if self.terms[i].coef.abs() < 1e-6 {
                    self.terms.remove(i);
                }
                return;
            } else if self.terms[i].var > var {
                self.terms.insert(i, Term { coef, var });
                return;
            }
        }
        self.terms.push(Term { coef, var });
    }

    /// Binds the result of the expression to a variable.
    /// Equivalent to `let a = var(); var.equals(expr); a`.
    pub fn bind(&self) -> VarId {
        let var = var();
        var.equals(self.clone());
        var
    }
}

impl From<VarId> for LinExpr {
    fn from(var: VarId) -> LinExpr {
        // If the variable is known or dependent, return the corresponding expression
        if let Some(expr) = var.expression() {
            expr
        } else {
            LinExpr {
                terms: smallvec![Term { coef: 1.0, var }],
                constant: 0.0,
            }
        }
    }
}

impl From<f64> for LinExpr {
    fn from(value: f64) -> LinExpr {
        LinExpr {
            terms: smallvec![],
            constant: value,
        }
    }
}

impl fmt::Debug for LinExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, t) in self.terms.iter().enumerate() {
            if i > 0 {
                if t.coef > 0.0 {
                    write!(f, " + ")?;
                } else {
                    write!(f, " - ")?;
                }
            }
            if t.coef.abs() != 1.0 {
                write!(f, "{}×", t.coef.abs())?;
            }
            write!(f, "v{}", t.var.0)?;
        }
        if self.terms.is_empty() {
            write!(f, "{}", self.constant)?;
        } else if self.constant != 0.0 {
            write!(f, " + {}", self.constant)?;
        }
        Ok(())
    }
}

/// Represents an equation of the form `a0 × x0 + ... + an × xn + b = 0`.
#[derive(Clone, PartialEq)]
struct Equation {
    /// The left-hand side of the equation (the `a0 × x0 + ... + an × xn + b` expression).
    expr: LinExpr,
}

impl Equation {
    /// Returns the expression that expresses a variable of the equation in terms of the others.
    /// Panics if the variable is not present in the equation.
    fn dependent_expr(&self, var: VarId) -> LinExpr {
        let mut expr = self.expr.clone();
        let ivar = expr
            .terms
            .iter()
            .position(|term| term.var == var)
            .expect("Variable not found in equation");
        let term = expr.terms.remove(ivar);
        expr = expr * (-1.0 / term.coef);
        expr
    }
}

impl fmt::Debug for Equation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} = 0", self.expr)
    }
}

/// Represents a system of linear equations.
#[derive(Clone, Debug)]
pub(super) struct System {
    vars: Vec<Variable>,
    equations: Vec<Equation>,
    dirty: bool,
}

impl System {
    pub(super) fn new() -> System {
        System {
            vars: Vec::new(),
            equations: Vec::new(),
            dirty: false,
        }
    }

    pub(super) fn var(&self, id: VarId) -> &Variable {
        &self.vars[id.0 as usize]
    }

    fn add_equation(&mut self, lhs: LinExpr, rhs: LinExpr) {
        let equation = Equation { expr: lhs - rhs };
        //dbg!(&equation);
        // find the first term with an independent variable
        let pos = equation
            .expr
            .terms
            .iter()
            .rposition(|term| self.vars[term.var.0 as usize].is_independent());
        if let Some(pos) = pos {
            // Deduce the expression to compute the variable from the others
            let mut expr = equation.expr.clone();
            let term = expr.terms.remove(pos);
            let var = term.var;
            expr = expr * (-1.0 / term.coef);
            // eliminate it from all other dependents
            for i in 0..self.vars.len() {
                if i != term.var.0 as usize {
                    match &mut self.vars[i] {
                        Variable::Dependent(de) => {
                            de.subst(var, &expr);
                        }
                        _ => {}
                    }
                }
            }
            self.vars[var.0 as usize] = Variable::Dependent(expr);
        } else {
            // all variables known or dependent
            panic!("All variables in equation are already dependent");
        }
    }

    pub(super) fn add_var(&mut self) -> VarId {
        assert!(self.vars.len() < u16::MAX as usize);
        self.vars.push(Variable::UnknownIndependent);
        VarId((self.vars.len() - 1) as u16)
    }
}

impl Mul<f64> for VarId {
    type Output = LinExpr;

    fn mul(self, coef: f64) -> LinExpr {
        LinExpr {
            terms: smallvec![Term { coef, var: self }],
            constant: 0.0,
        }
    }
}

impl Mul<VarId> for f64 {
    type Output = LinExpr;

    fn mul(self, var: VarId) -> LinExpr {
        LinExpr {
            terms: smallvec![Term { coef: self, var }],
            constant: 0.0,
        }
    }
}

impl Mul<f64> for LinExpr {
    type Output = LinExpr;

    fn mul(mut self, coef: f64) -> LinExpr {
        for term in &mut self.terms {
            term.coef *= coef;
        }
        self.constant *= coef;
        self
    }
}

impl Mul<LinExpr> for f64 {
    type Output = LinExpr;

    fn mul(self, mut expr: LinExpr) -> LinExpr {
        for term in &mut expr.terms {
            term.coef *= self;
        }
        expr.constant *= self;
        expr
    }
}

macro_rules! impl_add_sub {
    ($lhs:ty, $rhs:ty) => {
        impl Add<$rhs> for $lhs {
            type Output = LinExpr;

            fn add(self, other: $rhs) -> LinExpr {
                LinExpr::from(self) + LinExpr::from(other)
            }
        }

        impl Sub<$rhs> for $lhs {
            type Output = LinExpr;

            fn sub(self, other: $rhs) -> LinExpr {
                LinExpr::from(self) - LinExpr::from(other)
            }
        }
    };
}

impl_add_sub!(VarId, VarId);
impl_add_sub!(VarId, f64);
impl_add_sub!(f64, VarId);
impl_add_sub!(f64, LinExpr);
impl_add_sub!(LinExpr, f64);
impl_add_sub!(LinExpr, VarId);
impl_add_sub!(VarId, LinExpr);

impl Add<LinExpr> for LinExpr {
    type Output = LinExpr;

    fn add(mut self, other: LinExpr) -> LinExpr {
        for term in other.terms {
            self.add_term(term.coef, term.var);
        }
        self.constant += other.constant;
        self
    }
}

impl Sub<LinExpr> for LinExpr {
    type Output = LinExpr;

    fn sub(mut self, mut other: LinExpr) -> LinExpr {
        for term in other.terms {
            self.add_term(-term.coef, term.var);
        }
        self.constant -= other.constant;
        self
    }
}
