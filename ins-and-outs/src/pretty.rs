use super::syntax::*;
use crate::Closure;

// Copied from jankscripten
pub trait Pretty {
    fn pretty<'b, D, A>(&'b self, pp: &'b D) -> pretty::DocBuilder<'b, D, A>
    where
        D: pretty::DocAllocator<'b, A>,
        D::Doc: Clone,
        A: Clone;
}

pub const DEFAULT_WIDTH: usize = 80;

// Copied from jankscripten
#[macro_export]
macro_rules! impl_Display_Pretty {
    ($T:ty) => {
        impl std::fmt::Debug for $T {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let pp = pretty::BoxAllocator;
                let doc = self.pretty::<_, ()>(&pp);
                doc.1.render_fmt($crate::pretty::DEFAULT_WIDTH, f)
            }
        }
        impl std::fmt::Display for $T {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }
    };
}

////////////////////////////////////////////////////////////////////////////////

fn parens_if<'b, D, A, T>(pp: &'b D, d: &'b T, b: bool) -> pretty::DocBuilder<'b, D, A>
where
    T: Pretty,
    D: pretty::DocAllocator<'b, A>,
    A: std::clone::Clone,
    <D as pretty::DocAllocator<'b, A>>::Doc: std::clone::Clone,
{
    if b {
        pp.concat(vec![pp.text("("), d.pretty(pp), pp.text(")")])
    } else {
        d.pretty(pp)
    }
}

impl Pretty for Typ {
    fn pretty<'b, D, A>(&'b self, pp: &'b D) -> pretty::DocBuilder<'b, D, A>
    where
        D: pretty::DocAllocator<'b, A>,
        A: std::clone::Clone,
        <D as pretty::DocAllocator<'b, A>>::Doc: std::clone::Clone,
    {
        match self {
            Typ::Null => pp.text("null"),
            Typ::Arr(t1, t2) => pp.concat(vec![
                parens_if(pp, &**t1, t1.is_arr()),
                pp.space(),
                pp.text("->"),
                pp.space(),
                t2.pretty(pp),
            ]),
            Typ::Any => pp.text("any"),
            Typ::Metavar(i) => pp.text(greek(*i)),
            Typ::MetavarArg(t) => t.pretty(pp).append(pp.text("?")),
            Typ::MetavarRet(t) => t.pretty(pp).append(pp.text("!")),
        }
    }
}

/// produces lowercase greek letters in alphabetic order, then produced <i>
/// where i begins at 1 after the greek characters
fn greek(i: u32) -> String {
    let num_greek_chars = 78;
    if i <= num_greek_chars {
        // SAFETY:
        // - a char is a u32 if the u32 is a valid Unicode codepoint
        // - all characters between 0x03b1 and 0x03ff are greek characters
        // - if i is 78, total is 0x03ff, if i is 0, it is 0x03b1
        // - so all integers produced by this path are Unicode code points
        // 0x03b1 is α
        unsafe { std::mem::transmute::<u32, char>(0x03b1 + i) }.to_string()
    } else {
        format!("⦉{}⦊", i - num_greek_chars)
    }
}

impl Pretty for Exp {
    fn pretty<'b, D, A>(&'b self, pp: &'b D) -> pretty::DocBuilder<'b, D, A>
    where
        D: pretty::DocAllocator<'b, A>,
        A: std::clone::Clone,
        <D as pretty::DocAllocator<'b, A>>::Doc: std::clone::Clone,
    {
        match self {
            Exp::Null => pp.text("null"),
            Exp::Var(x) => pp.text(x),
            Exp::Assign(x, v) => pp.concat(vec![pp.text(x), pp.text(": "), v.pretty(pp)]),
            Exp::Fun(x, t1, e, t2) => pp.concat(vec![
                pp.text("fun"),
                pp.space(),
                pp.text(x),
                pp.text(":"),
                t1.pretty(pp),
                pp.text("."),
                pp.line(),
                e.pretty(pp).nest(2),
                pp.text(":"),
                t2.pretty(pp),
            ]),
            Exp::App(e1, e2) => pp.concat(vec![
                parens_if(pp, &**e1, e1.is_fun_exp()),
                pp.space(),
                parens_if(pp, &**e2, e2.is_atom() == false),
            ]),
            Exp::If(e1, e2, e3) => pp.concat(vec![
                pp.text("if"),
                pp.space(),
                e1.pretty(pp).nest(2),
                pp.line(),
                pp.concat(vec![pp.text("then"), pp.line(), e2.pretty(pp)])
                    .nest(2),
                pp.line(),
                pp.concat(vec![pp.text("else"), pp.line(), e3.pretty(pp)])
                    .nest(2),
            ]),
            Exp::Coerce(t1, t2, e) => pp.concat(vec![
                pp.text("⟨"),
                pretty_coercion(t1, t2, pp),
                pp.text("⟩"),
                pp.space(),
                parens_if(pp, &**e, e.is_fun_exp()),
            ]),
        }
    }
}

fn pretty_coercion<'b, D, A>(t1: &'b Typ, t2: &'b Typ, pp: &'b D) -> pretty::DocBuilder<'b, D, A>
where
    D: pretty::DocAllocator<'b, A>,
    A: std::clone::Clone,
    <D as pretty::DocAllocator<'b, A>>::Doc: std::clone::Clone,
{
    pp.concat(vec![
        t1.pretty(pp),
        pp.space(),
        pp.text("▷"),
        pp.space(),
        t2.pretty(pp),
    ])
}

impl Pretty for (Typ, Typ) {
    fn pretty<'b, D, A>(&'b self, pp: &'b D) -> pretty::DocBuilder<'b, D, A>
    where
        D: pretty::DocAllocator<'b, A>,
        A: std::clone::Clone,
        <D as pretty::DocAllocator<'b, A>>::Doc: std::clone::Clone,
    {
        pp.concat(vec![
            self.0.pretty(pp),
            pp.space(),
            pp.text("▷"),
            pp.space(),
            self.1.pretty(pp),
        ])
    }
}

impl Pretty for Closure {
    fn pretty<'b, D, A>(&'b self, pp: &'b D) -> pretty::DocBuilder<'b, D, A>
    where
        D: pretty::DocAllocator<'b, A>,
        A: std::clone::Clone,
        <D as pretty::DocAllocator<'b, A>>::Doc: std::clone::Clone,
    {
        pp.intersperse(
            self.iter().map(|(t1, t2)| pretty_coercion(t1, t2, pp)),
            pp.text(",").append(pp.softline()),
        )
    }
}

pub struct DisplayClosure<'a>(pub &'a Closure);
impl Pretty for DisplayClosure<'_> {
    fn pretty<'b, D, A>(&'b self, pp: &'b D) -> pretty::DocBuilder<'b, D, A>
    where
        D: pretty::DocAllocator<'b, A>,
        A: std::clone::Clone,
        <D as pretty::DocAllocator<'b, A>>::Doc: std::clone::Clone,
    {
        self.0.pretty(pp)
    }
}

impl_Display_Pretty!(Typ);
impl_Display_Pretty!(Exp);
impl_Display_Pretty!(DisplayClosure<'_>);
