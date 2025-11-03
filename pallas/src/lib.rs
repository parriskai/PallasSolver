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

fn identity<'a>(input: &'a str) -> PResult<'a, String>{
    And(
        AnyWhere(|c: char| c.is_alphabetic() || c == '_'),
        Optional(AnyWhere(|c:char | c.is_alphanumeric())))
        .invoke(input)
        .map(|(res, (a, b)) | {
            if let Some(b) = b{
                return (res, a.to_string() + b);
            } else {
                return (res, a.to_string());
            }
    })
}

fn internal_name<'a>(input: &'a str) -> PResult<'a, String>{
    let (input, _) = Exact("$".into()).invoke(input)?;
    let (res, name) = identity(input)?;
    Ok((res, name))
}

fn define<'a>(input: &'a str) -> PResult<'a, ()>{
    let (input, _) = Exact("define".into()).invoke(input)?;
    let (input, _) = whitespace1(input)?;
    let (input, name) = identity(input)?;
    let (input, _) = whitespace0(input)?;
}