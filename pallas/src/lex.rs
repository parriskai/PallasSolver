use rlex::{AnyWhere, MinChars, Optional, PResult, Parser, Punctuated, utils::whitespace1};

pub mod keywords{
    use rlex::ExactStaticStr;

    pub static AS: ExactStaticStr = ExactStaticStr("as");
    pub static USE: ExactStaticStr = ExactStaticStr("use");
}

pub mod symbols{
    use rlex::ExactStaticStr;

    pub static STAR: ExactStaticStr = ExactStaticStr("*");
    pub static DOUBLE_COLON: ExactStaticStr = ExactStaticStr("::");
    pub static LEFT_CB: ExactStaticStr = ExactStaticStr("{");
    pub static RIGHT_CB: ExactStaticStr = ExactStaticStr("}");
    pub static COMMA: ExactStaticStr = ExactStaticStr(",");
}

macro_rules! ws_wrapped {
    ($t: expr) => {(::rlex::utils::whitespace0, $t, ::rlex::utils::whitespace0).map(|(_,v,_)|v)};
}

#[derive(Debug)]
pub struct Name(pub String);

pub fn name<'a>(input: &'a str) -> PResult<'a, Name>{
    (
        MinChars(AnyWhere(|c: char| c.is_alphabetic() || c == '_'), 1),
        Optional(AnyWhere(|c:char | c.is_alphanumeric()))
    ).invoke(input)
     .map(|(res, (a, b)) | {
            if let Some(b) = b{
                return (res, Name(a.to_string() + b));
            } else {
                return (res, Name(a.to_string()));
            }
    })
}

#[derive(Debug)]
pub enum Path {
    WildCard,
    PathAs(Box<Path>, Name),
    MultiPath(Vec<Path>),
    Ordinary(Name, Option<Box<Path>>)
}

fn single_path_ordinary<'a>(input: &'a str) -> PResult<'a, Path>{
    (name, Optional((symbols::DOUBLE_COLON, single_path_ordinary))).map(|(n, o)| Path::Ordinary(n, o.map(|(_,p)|Box::new(p)))).invoke(input)
}

fn path_ordinary<'a>(input: &'a str) -> PResult<'a, Path>{
    (name, Optional((symbols::DOUBLE_COLON, path_not_as))).map(|(n, o)| Path::Ordinary(n, o.map(|(_,p)|Box::new(p)))).invoke(input)
}

fn multi_path<'a>(input: &'a str) -> PResult<'a, Path>{
    (
        symbols::LEFT_CB,
        ws_wrapped!(Punctuated(path,ws_wrapped!(symbols::COMMA)).map(|i| i.into_iter().map(|x| x.0).collect())),
        symbols::RIGHT_CB
    ).map(|(_, l, _)| Path::MultiPath(l)).invoke(input)
}

fn path_not_as<'a>(input: &'a str) -> PResult<'a, Path>{
    path_ordinary.or(multi_path).or(symbols::STAR.map(|_| Path::WildCard)).invoke(input)
}

fn path_as<'a>(input: &'a str) -> PResult<'a, Path>{
    (single_path_ordinary, whitespace1, keywords::AS, whitespace1, name).map(|(p, _, _, _, n)| Path::PathAs(Box::new(p), n)).invoke(input)
}

pub fn path<'a>(input: &'a str) -> PResult<'a, Path>{
    symbols::STAR.map(|_| Path::WildCard)
        .or(path_as)
        .or(path_ordinary)
        .or(multi_path)
        .invoke(input)
}