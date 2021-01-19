#[derive(Debug, PartialEq, Clone)]
pub enum Typ {
    Int,
    Bool,
    Str,
    Arr(Box<Typ>, Box<Typ>),
    Metavar(u32),
}

impl Typ {
    pub fn expect_metavar(&self) -> u32 {
        match self {
            Typ::Metavar(n) => *n,
            _ => panic!("expected a Typ::Metavar"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Lit {
    Int(i32),
    Bool(bool),
    Str(String),
}

impl Lit {
    pub fn typ(&self) -> Typ {
        match self {
            Lit::Int(_) => Typ::Int,
            Lit::Bool(_) => Typ::Bool,
            Lit::Str(_) => Typ::Str,
        }
    }
}

pub type Id = String;

#[derive(Debug, PartialEq, Clone)]
pub enum Exp {
    Lit(Lit),
    Var(Id),
    Fun(Id, Typ, Box<Exp>),
    App(Box<Exp>, Box<Exp>),
    Add(Typ, Box<Exp>, Box<Exp>),
}
