use rlex::{And, AnyWhere, AnyWhile, Exact, Optional, PResult, Parser};

fn whitespace0<'a>(input: &'a str) -> PResult<'a, ()>{
    let (res, w) = AnyWhere(|c: char| c.is_whitespace()).invoke(input)?;
    return Ok((res, ()));
}

fn whitespace1<'a>(input: &'a str) -> PResult<'a, ()>{
    let (res, w) = AnyWhere(|c: char| c.is_whitespace()).invoke(input)?;
    if w.len() == 0{
        return Err(());
    } else{
        return Ok((res, ()));
    }
}

pub struct Identifier(pub String);

fn identity<'a>(input: &'a str) -> PResult<'a, Identifier>{
    And(
        AnyWhere(|c: char| c.is_alphabetic() || c == '_'),
        Optional(AnyWhere(|c:char | c.is_alphanumeric())))
        .invoke(input)
        .map(|(res, (a, b)) | {
            if let Some(b) = b{
                return (res, Identifier(a.to_string() + b));
            } else {
                return (res, Identifier(a.to_string()));
            }
    })
}

pub struct InternalName(pub Identifier);

fn internal_name<'a>(input: &'a str) -> PResult<'a, InternalName>{
    let (input, _) = Exact("$".into()).invoke(input)?;
    let (res, name) = identity(input)?;
    Ok((res, InternalName(name)))
}

pub struct InternalValue{pub name: InternalName, pub value: Box<Expr>}

fn internal_value<'a>(input: &'a str) -> PResult<'a, InternalValue>{
    let (input, name) = internal_name(input)?;
    let (input, _) = whitespace0(input)?;
    let (input, _) = Exact("{".into()).invoke(input)?;
    let (input, (data, _)) = AnyUntil(|a: &str, b: &str| {
        let (res, _) = Exact("}".into()).invoke(b)?;
        Ok((res, ()))
    })?;
    Ok((res, InternalValue{name, value: Box::new(value)}))
}

pub enum Expr{
    InternalValue(InternalValue),
    Variable(Identifier),
}

fn expr<'a>(input: &'a str) -> PResult<'a, Expr>{
    Or(
        internal_value(input).map(|(res, v)| (res, Expr::InternalValue(v))),
        identity(input).map(|(res, v)| (res, Expr::Variable(v)))
    )
}

pub struct Definition{pub name: Identifier, pub value: Expr};

fn define<'a>(input: &'a str) -> PResult<'a, ()>{
    let (input, _) = Exact("define".into()).invoke(input)?;
    let (input, _) = whitespace1(input)?;
    let (input, name) = identity(input)?;
    let (input, _) = whitespace0(input)?;
    let (input, value) = expr(input)?;
    let (result, _) = Exact(";".into()).invoke(input)?;
    Ok((result, ())
}