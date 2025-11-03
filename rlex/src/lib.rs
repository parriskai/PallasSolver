pub type PResult<'a, T> = core::result::Result<(&'a str, T), ()>;

pub trait Parser<'a, T> {
    fn invoke(&self, input: &'a str) -> PResult<'a, T>;
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

pub struct And<A,B>(pub A, pub B);

impl<'a, AT, BT, A: Parser<'a, AT>, B: Parser<'a, BT>> Parser<'a, (AT, BT)> for And<A,B> {
    fn invoke(&self, input: &'a str) -> PResult<'a, (AT, BT)> {
        let (input, a) = self.0.invoke(input)?;
        let (result, b) = self.1.invoke(input)?;
        Ok((result, (a, b)))
    }
}