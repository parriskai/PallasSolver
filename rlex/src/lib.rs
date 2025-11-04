use std::marker::PhantomData;

pub type PResult<'a, T> = core::result::Result<(&'a str, T), ()>;

pub trait Parser<'a, T> {
    fn invoke(&self, input: &'a str) -> PResult<'a, T>;
    fn map<K, F: Fn(T) -> K>(self, f: F) -> Map<Self, T, F> where Self: Sized{
        Map {inner: self, f, _phantom: PhantomData}
    }
}

pub struct Map<P, T, F>{
    inner: P,
    f: F,
    _phantom: PhantomData<T>
}

impl<'a, P, O, T, F> Parser<'a, O> for Map<P, T, F> where P: Parser<'a, T>, F: Fn(T) -> O{
    fn invoke(&self, input: &'a str) -> PResult<'a, O> {
        self.inner.invoke(input).map(|(r, t)| (r, (self.f)(t)))
    }
}

impl<'a, T, F> Parser<'a, T> for F
where
    F: Fn(&'a str) -> PResult<'a, T>,
{
    fn invoke(&self, input: &'a str) -> PResult<'a, T> {
        self(input)
    }
}


pub struct Exact(pub String);
impl<'a> Parser<'a, ()> for Exact {
    fn invoke(&self, input: &'a str) -> PResult<'a, ()> {
        if input.starts_with(self.0.as_str()){
            Ok((&input[self.0.len()..],()))
        } else {Err(())}
    }
}

pub struct AnyN(pub usize);

impl<'a> Parser<'a, &'a str> for AnyN {
    fn invoke(&self, input: &'a str) -> PResult<'a, &'a str> {
        if input.len() >= self.0 {
            let (a, b) = input.split_at(self.0);
            Ok((b, a))
        } else {
            Err(())
        }
    }
}


pub struct AnyUntil<F>(pub F);

impl<'a, T, F> Parser<'a, (&'a str, T)> for AnyUntil<F>
where
    F: Fn(&'a str, &'a str) -> PResult<'a, T>,
{
    fn invoke(&self, input: &'a str) -> PResult<'a, (&'a str, T)> {
        for (loc, _) in input.char_indices() {
            let (a, b) = input.split_at(loc);
            if let Ok((res, t)) = (self.0)(a, b) {
                return Ok((res, (a, t)));
            }
        }
        Err(())
    }
}

pub struct AnyWhile(pub fn(&str, &str) -> bool);

impl<'a> Parser<'a, &'a str> for AnyWhile {
    fn invoke(&self, input: &'a str) -> PResult<'a, &'a str> {
        let mut last = 0;
        for (i, _) in input.char_indices() {
            let (consumed, remaining) = input.split_at(i);
            if !(self.0)(consumed, remaining) {
                if i == 0 { return Err(()); }
                let (consumed, remaining) = input.split_at(last);
                return Ok((remaining, consumed));
            }
            last = i;
        }
        Ok(("", input))
    }
}

pub struct AnyWhere<F>(pub F);

impl<'a, F> Parser<'a, &'a str> for AnyWhere<F>
where
    F: Fn(char) -> bool,
{
    fn invoke(&self, input: &'a str) -> PResult<'a, &'a str> {
        let mut end = 0;
        for (i, ch) in input.char_indices() {
            if (self.0)(ch) {
                end = i + ch.len_utf8();
            } else {
                break;
            }
        }
        let (consumed, remaining) = input.split_at(end);
        Ok((remaining, consumed))
    }
}

pub struct Optional<P>(pub P);

impl <'a, T, P: Parser<'a, T>> Parser<'a, Option<T>> for Optional<P>{
    fn invoke(&self, input: &'a str) -> PResult<'a, Option<T>> {
        if let Ok((res, t)) = self.0.invoke(input){
            return Ok((res, Some(t)));
        }
        else {
            return Ok((input, None));
        }
    }
}

pub struct Or<A, B>(pub A, pub B);

impl<'a, T, A: Parser<'a, T>, B: Parser<'a, T>> Parser<'a, T> for Or<A,B> {
    fn invoke(&self, input: &'a str) -> PResult<'a, T> {
        if let Ok((res, t)) = self.0.invoke(input){
            return Ok((res, t));
        }
        else {
            return self.1.invoke(input);
        }
    }
}

pub fn whitespace0<'a>(input: &'a str) -> PResult<'a, ()>{
    let (res, _) = AnyWhere(|c: char| c.is_whitespace()).invoke(input)?;
    return Ok((res, ()));
}

pub fn whitespace1<'a>(input: &'a str) -> PResult<'a, ()>{
    let (res, w) = AnyWhere(|c: char| c.is_whitespace()).invoke(input)?;
    if w.len() == 0{
        return Err(());
    } else{
        return Ok((res, ()));
    }
}

pub trait MultiInvoke<'a, T> {
    fn _and(&self, input: &'a str) -> PResult<'a, T>;
    fn _or(&self, input: &'a str) -> PResult<'a, T>;
}