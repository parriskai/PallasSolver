use std::marker::PhantomData;
use concat_idents::concat_idents;
pub mod utils;

/// Parser result type.
/// 'a: the lifetime of the input
/// T: the type of the parsed value
/// Wrapper for Result<(&str, Value), Error>
pub type PResult<'a, T> = core::result::Result<(&'a str, T), ()>;

/// Parser trait
/// 'a: the lifetime of the input
/// T: the type of the parsed value
pub trait Parser<'a, T> {
    /// Invoke the parser on the input
    /// input: the input to parse
    /// returns: the remaining input and the parsed value if successful, otherwise an error
    fn invoke(&self, input: &'a str) -> PResult<'a, T>;
    
    /// Map the parsed value to a new type
    /// K: the type of the new value
    /// F: the function to map the value
    /// returns: a new parser that maps the parsed value to a new type
    fn map<K, F: Fn(T) -> K>(self, f: F) -> Map<Self, T, F> where Self: Sized{
        Map {inner: self, f, _phantom: PhantomData}
    }

    /// Union of two parsers
    /// P: the type of the second parser
    /// returns: An Or parser that tries the first parser, and if it fails, tries the second parser
    /// If both parsers fail, returns an error
    //// Can be chained
    fn or<P: Parser<'a, T>>(self, other: P) -> Or<Self, P> where Self: Sized{
        Or(self, other)
    }
}

/// Mapped Parser Type
/// P: the inner parser
/// T: the type of the parsed value
/// F: the function to map the value
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

/// Or Parser Type
/// A: the first parser
/// B: the second parser
/// T: the type of the parsed value
/// If the first parser succeeds, returns its value
/// If the first parser fails, tries the second parser
/// If both parsers fail, returns an error
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

// Implement Parser for generic functions
impl<'a, T, F> Parser<'a, T> for F
where F: Fn(&'a str) -> PResult<'a, T>{
    fn invoke(&self, input: &'a str) -> PResult<'a, T> {
        self(input)
    }
}

/// Exact String type Parser
/// String: the string to consume
/// Consumes exactly the string given.
/// If the input starts with the string, returns a blank Ok value.
/// If the input does not start with the string, returns an error.
pub struct Exact(pub String);

impl<'a> Parser<'a, ()> for Exact {
    fn invoke(&self, input: &'a str) -> PResult<'a, ()> {
        if input.starts_with(self.0.as_str()){
            Ok((&input[self.0.len()..],()))
        } else {Err(())}
    }
}


/// AnyN Parser
/// usize: the number of characters to consume
/// Consumes exactly N characters.
/// If the input is at least N characters long, consumes N characters and returns them as a &str.
/// If the input is less than N characters long, returns an error.
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

/// Any Until Parser
/// P: Terminating parser
/// Consumes until the terminating parser succeeds.
/// If the terminating parser succeeds, returns a &str and terminating value.
/// If the terminating parser never succeeds, returns an error.
pub struct AnyUntil<P>(pub P);

impl<'a, T, P> Parser<'a, (&'a str, T)> for AnyUntil<P>
where
    P: Parser<'a, T>
{
    fn invoke(&self, input: &'a str) -> PResult<'a, (&'a str, T)> {
        for (loc, _) in input.char_indices() {
            let (a, b) = input.split_at(loc);
            if let Ok((res, t)) = self.0.invoke(b) {
                return Ok((res, (a, t)));
            }
        }
        Err(())
    }
}

/// Any While Parser
/// fn: consumed condition function
/// Consumes while the condition function returns true.
/// When the condition function returns false, returns a &str of all characters until the first false.
/// If the condition function never returns false, consumes and returns the entire input.
/// Never returns an error.
pub struct AnyWhile(pub fn(&str, &str) -> bool);

impl<'a> Parser<'a, &'a str> for AnyWhile {
    fn invoke(&self, input: &'a str) -> PResult<'a, &'a str> {
        let mut last = 0;
        for (i, _) in input.char_indices() {
            let (consumed, remaining) = input.split_at(i);
            if !(self.0)(consumed, remaining) {
                if i == 0 { return Ok((input, "")) }
                let (consumed, remaining) = input.split_at(last);
                return Ok((remaining, consumed));
            }
            last = i;
        }
        Ok(("", input))
    }
}

/// Any Where Parser
/// fn: character condition function
/// Consumes every character that satisfies the condition function until the first character that does not.
/// If the condition function never returns false, consumes and returns the entire input.
/// Never returns an error.
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


/// Min Chars Parser
/// P: the parser to consume
/// usize: the minimum number of characters to consume
/// If the parser succeeds and consumes at least N characters, returns the parsed value.
/// If the parser succeeds and consumes less than N characters, returns an error.
/// If the parser fails, returns an error.
pub struct MinChars<P>(pub P, pub usize);

impl<'a, T, P: Parser<'a, T>> Parser<'a, T> for MinChars<P> {
    fn invoke(&self, input: &'a str) -> PResult<'a, T> {
        let (remainder, t) = self.0.invoke(input)?;
        if input.chars().count() - remainder.chars().count() >= self.1{
            Ok((remainder, t))
        } else{
            Err(())
        }
    }
}

/// Optional Parser
/// P: the parser to optionally consume
/// If the parser succeeds, returns the parsed value.
/// If the parser fails, returns None.
/// Never returns an error.
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

macro_rules! tuple_and_parser {
    ($($name:ident:$pn:ident),+) => {
        #[allow(nonstandard_style)]
        impl <'a, $($name, $pn: Parser<'a, $name>),+> Parser<'a, ($($name),+)> for ($($pn),+){
            fn invoke(&self, input: &'a str) -> PResult<'a,  ($($name),+)>{
                let ($($name),+) = self;
                $(
                    let (input, $name) = $name.invoke(input)?;

                )+
                return Ok((input, ($($name),+)))
            }
        }
    };
}

tuple_and_parser!(A:PA, B:PB);
tuple_and_parser!(A:PA, B:PB, C:PC);
tuple_and_parser!(A:PA, B:PB, C:PC, D:PD);
tuple_and_parser!(A:PA, B:PB, C:PC, D:PD, E:PE);
tuple_and_parser!(A:PA, B:PB, C:PC, D:PD, E:PE, F:PF);
tuple_and_parser!(A:PA, B:PB, C:PC, D:PD, E:PE, F:PF, G:PG);
tuple_and_parser!(A:PA, B:PB, C:PC, D:PD, E:PE, F:PF, G:PG, H:PH);


/// Repetition Parser
/// P: the parser to repeat
/// Consumes as many times as possible.
/// When the parser fails, returns a Vec of all parsed values.
/// Never returns an error.
pub struct Repetition<P>(pub P);
impl<'a, T, P: Parser<'a, T>> Parser<'a, Vec<T>> for Repetition<P>{
    fn invoke(&self, input: &'a str) -> PResult<'a, Vec<T>>{
        let mut res = Vec::new();
        let mut input = input;
        while let Ok((rem, t)) = self.0.invoke(input){
            res.push(t);
            input = rem;
        }
        Ok((input, res))
    }
}


/// Min Repition Parser
/// P: the parser to repeat
/// usize: the minimum number of times to repeat
/// Consumes at least N times.
/// If the parser succeeds and consumes at least N times, returns a Vec of all parsed values.
/// Otherwise, returns an error.
pub struct MinRepetition<P>(pub P, pub usize);
impl<'a, T, P: Parser<'a, T>> Parser<'a, Vec<T>> for MinRepetition<P>{
    fn invoke(&self, input: &'a str) -> PResult<'a, Vec<T>>{
        let mut res = Vec::new();
        let mut input = input;
        while let Ok((rem, t)) = self.0.invoke(input){
            res.push(t);
            input = rem;
        }
        if res.len() >= self.1{
            Ok((input, res))
        } else{
            Err(())
        }
    }
}

/// MinMax Repition Parser
/// P: the parser to repeat
/// (Option<usize>, Option<usize>): the minimum and maximum number of times to repeat)
/// Consumes at least N times and at most M times.
/// If the parser succeeds and consumes at least N times and at most M times, returns a Vec of all parsed values.
/// Otherwise, returns an error.
pub struct MinMaxRepetition<P>(pub P, pub (Option<usize>, Option<usize>));
impl<'a, T, P: Parser<'a, T>> Parser<'a, Vec<T>> for MinMaxRepetition<P>{
    fn invoke(&self, input: &'a str) -> PResult<'a, Vec<T>>{
        let mut res = Vec::new();
        let mut input = input;
        while let Ok((rem, t)) = self.0.invoke(input){
            res.push(t);
            input = rem;
        }
        if let Some(min) = self.1.0{
            if res.len() < min{
                return Err(());
            }
        }
        if let Some(max) = self.1.1{
            if res.len() > max{
                return Err(());
            }
        }
        Ok((input, res))
    }
}