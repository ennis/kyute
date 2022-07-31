use crate::{style::WidgetState, Atom, Environment, LayoutParams};
use cssparser::{ParseError, Parser, Token};
use std::sync::Arc;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Pseudoclass {
    Hover,
    Focus,
    Active,
    Disabled,
}

pub(crate) enum Predicate {
    Env(Atom),
    State(WidgetState),
    Or(Arc<Predicate>, Arc<Predicate>),
    And(Arc<Predicate>, Arc<Predicate>),
    Not(Arc<Predicate>),
}

impl Predicate {
    pub(crate) fn eval(&self, state: WidgetState, constraints: &LayoutParams, env: &Environment) -> bool {
        match self {
            Predicate::Env(var_name) => {
                let value: Option<bool> = env.get_by_name(var_name);
                if let Some(value) = value {
                    value
                } else {
                    warn!(
                        "style environment variable `${}` does not exist or does not have the correct type",
                        var_name
                    );
                    false
                }
            }
            Predicate::State(s) => state.contains(*s),
            Predicate::Or(a, b) => a.eval(state, constraints, env) || a.eval(state, constraints, env),
            Predicate::And(a, b) => a.eval(state, constraints, env) && a.eval(state, constraints, env),
            Predicate::Not(a) => !a.eval(state, constraints, env),
        }
    }

    pub(crate) fn variant_states(&self) -> WidgetState {
        match *self {
            Predicate::State(state) => state,
            _ => WidgetState::empty(),
        }
    }
}

fn parse_predicate_term<'i>(input: &mut Parser<'i, '_>) -> Result<Predicate, ParseError<'i, ()>> {
    match input.next()? {
        Token::Delim('$') => {
            let var = input.expect_ident()?;
            Ok(Predicate::Env(Atom::from(&**var)))
        }
        Token::Colon => {
            let pseudoclass = input.expect_ident()?.clone();
            match &*pseudoclass {
                "active" => Ok(Predicate::State(WidgetState::ACTIVE)),
                "focus" => Ok(Predicate::State(WidgetState::FOCUS)),
                "hover" => Ok(Predicate::State(WidgetState::HOVER)),
                "disabled" => Ok(Predicate::State(WidgetState::DISABLED)),
                _ => {
                    return Err(input.new_unexpected_token_error(Token::Ident(pseudoclass)));
                }
            }
        }
        token => {
            let token = token.clone();
            return Err(input.new_unexpected_token_error(token));
        }
    }
}

fn parse_predicate_negation<'i>(input: &mut Parser<'i, '_>) -> Result<Predicate, ParseError<'i, ()>> {
    let neg = input.try_parse(|input| input.expect_delim('!')).is_ok();
    let term = parse_predicate_term(input)?;
    if neg {
        Ok(Predicate::Not(Arc::new(term)))
    } else {
        Ok(term)
    }
}

fn parse_predicate_conjunction<'i>(input: &mut Parser<'i, '_>) -> Result<Predicate, ParseError<'i, ()>> {
    let lhs = parse_predicate_negation(input)?;
    if input.is_exhausted() {
        return Ok(lhs);
    }
    input.expect_delim('|')?;
    input.expect_delim('|')?;
    let rhs = parse_predicate_negation(input)?;
    Ok(Predicate::Or(Arc::new(lhs), Arc::new(rhs)))
}

fn parse_predicate_disjunction<'i>(input: &mut Parser<'i, '_>) -> Result<Predicate, ParseError<'i, ()>> {
    let lhs = parse_predicate_conjunction(input)?;
    if input.is_exhausted() {
        return Ok(lhs);
    }
    input.expect_delim('&')?;
    input.expect_delim('&')?;
    let rhs = parse_predicate_conjunction(input)?;
    Ok(Predicate::Or(Arc::new(lhs), Arc::new(rhs)))
}

pub(crate) fn parse_predicate<'i>(input: &mut Parser<'i, '_>) -> Result<Predicate, ParseError<'i, ()>> {
    parse_predicate_disjunction(input)
}
