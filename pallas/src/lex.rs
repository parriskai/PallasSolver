use rlex::{AnyUntil, AnyWhere, Exact, MinChars, Optional, Or, PResult, Parser, utils::{whitespace0,whitespace1}};

#[derive(Debug)]
pub struct Identifier(pub String);

pub fn identity<'a>(input: &'a str) -> PResult<'a, Identifier>{
    println!("");
    (
        MinChars(AnyWhere(|c: char| c.is_alphabetic() || c == '_'), 1),
        Optional(AnyWhere(|c:char | c.is_alphanumeric()))
    ).invoke(input)
     .map(|(res, (a, b)) | {
            if let Some(b) = b{
                return (res, Identifier(a.to_string() + b));
            } else {
                return (res, Identifier(a.to_string()));
            }
    })
}

#[derive(Debug)]
pub struct InternalName(pub Identifier);

pub fn internal_name<'a>(input: &'a str) -> PResult<'a, InternalName>{
    let (input, _) = Exact("$".into()).invoke(input)?;
    let (res, name) = identity(input)?;
    Ok((res, InternalName(name)))
}

#[derive(Debug)]
pub struct InternalValue{pub name: InternalName, pub value: (String, Option<Box<Expr>>)}

pub fn internal_value<'a>(input: &'a str) -> PResult<'a, InternalValue>{
    let (input, name) = internal_name(input)?;
    let (input, _) = whitespace0(input)?;
    let (input, _) = Exact("{".into()).invoke(input)?;
    let (res, (data, _)) = AnyUntil(Exact("}".into())).invoke(input)?;

    let data = if let Ok((r, e)) = expr(data){
        if r.is_empty(){
            (data.to_string(), Some(Box::new(e)))
        }
        else {
            (data.to_string(), None)
        }
    } else {
        (data.to_string(), None)
    };

    Ok((res, InternalValue{name, value: data}))
}

#[derive(Debug)]
pub enum Expr{
    InternalValue(InternalValue),
    Variable(Identifier),
}

pub fn expr<'a>(input: &'a str) -> PResult<'a, Expr>{
    Or(
        internal_value.map(|v| Expr::InternalValue(v)),
        identity.map(|v| Expr::Variable(v))
    ).invoke(input)
}

#[derive(Debug)]
pub struct Definition{pub name: Identifier, pub value: Expr}

pub fn define<'a>(input: &'a str) -> PResult<'a, Definition>{
    let (input, _) = Exact("define".into()).invoke(input)?;
    let (input, _) = whitespace1(input)?;
    let (input, name) = identity(input)?;
    let (input, _) = whitespace0(input)?;
    let (input, _) = Exact("=".into()).invoke(input)?;
    let (input, _) = whitespace0(input)?;
    let (input, value) = expr(input)?;
    let (result, _) = Exact(";".into()).invoke(input)?;
    Ok((result, Definition{name, value}))
}