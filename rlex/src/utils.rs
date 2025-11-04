/// Consumes any single character
/// Returns the character consumed
/// If the input is empty, returns an error
pub fn any<'a>(input: &'a str) -> PResult<'a, char>{
    let ic = input.chars();
    if let Some(c = ic.next(){
        return (ic.as_str(), c)
    }
}

/// Consumes zero or more whitespace characters
/// Returns an empty Ok value
/// Never returns an error
pub fn whitespace0<'a>(input: &'a str) -> PResult<'a, ()>{
  let (res, _) = AnyWhere(|c: char| c.is_whitespace()).invoke(input)?;
  return Ok((res, ()));
}

/// Consumes one or more whitespace characters
/// Returns an empty Ok value
/// If the input does not start with a whitespace character, returns an error
pub fn whitespace1<'a>(input: &'a str) -> PResult<'a, ()>{
  let (res, w) = AnyWhere(|c: char| c.is_whitespace()).invoke(input)?;
  if w.len() == 0{
      return Err(());
  } else{
      return Ok((res, ()));
  }
}