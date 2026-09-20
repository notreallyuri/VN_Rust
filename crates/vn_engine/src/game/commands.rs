use std::collections::{BTreeMap, HashMap};

use vn_script::{CommandSig, ParamKind};

use crate::context::GameContext;
use crate::screen::ScreenState;

type Handler = Box<dyn Fn(&mut GameContext, &[String]) -> Option<ScreenState>>;

pub trait FromArg: Sized {
    const KIND: ParamKind;
    fn from_arg(arg: &str) -> Option<Self>;
}

macro_rules! from_arg {
    ($kind:ident: $($ty:ty),*) => {$(
        impl FromArg for $ty {
            const KIND: ParamKind = ParamKind::$kind;
            fn from_arg(arg: &str) -> Option<Self> {
                arg.parse().ok()
            }
        }
    )*};
}

from_arg!(Int: i8, i16, i32, i64);
from_arg!(UInt: u8, u16, u32, u64, usize);
from_arg!(Float: f32, f64);
from_arg!(Bool: bool);
from_arg!(Word: String);

pub trait Arg: Sized {
    const KIND: ParamKind;
    const OPTIONAL: bool;
    fn parse(arg: Option<&str>) -> Result<Self, String>;
}

macro_rules! arg {
    ($($ty:ty),*) => {$(
        impl Arg for $ty {
            const KIND: ParamKind = <$ty as FromArg>::KIND;
            const OPTIONAL: bool = false;
            fn parse(arg: Option<&str>) -> Result<Self, String> {
                let arg = arg.ok_or_else(|| "missing argument".to_string())?;
                <$ty>::from_arg(arg)
                    .ok_or_else(|| format!("`{}` is not {}", arg, <$ty as FromArg>::KIND))
            }
        }

        impl Arg for Option<$ty> {
            const KIND: ParamKind = <$ty as FromArg>::KIND;
            const OPTIONAL: bool = true;
            fn parse(arg: Option<&str>) -> Result<Self, String> {
                arg.map(|arg| <$ty as Arg>::parse(Some(arg))).transpose()
            }
        }
    )*};
}

arg!(
    i8, i16, i32, i64, u8, u16, u32, u64, usize, f32, f64, bool, String
);

pub trait FromArgs: Sized {
    fn signature() -> CommandSig;
    fn from_args(args: &[String]) -> Result<Self, String>;
}

impl FromArgs for () {
    fn signature() -> CommandSig {
        CommandSig::default()
    }

    fn from_args(_: &[String]) -> Result<Self, String> {
        Ok(())
    }
}

impl FromArgs for Vec<String> {
    fn signature() -> CommandSig {
        CommandSig {
            rest: Some(ParamKind::Word),
            ..CommandSig::default()
        }
    }

    fn from_args(args: &[String]) -> Result<Self, String> {
        Ok(args.to_vec())
    }
}

fn push_param(sig: &mut CommandSig, kind: ParamKind, optional: bool) {
    if optional {
        sig.optional.push(kind);
    } else {
        assert!(
            sig.optional.is_empty(),
            "command arguments: required arguments must come before optional ones"
        );
        sig.required.push(kind);
    }
}

macro_rules! from_args_tuple {
    ($($name:ident),+) => {
        impl<$($name: Arg),+> FromArgs for ($($name,)+) {
            fn signature() -> CommandSig {
                let mut sig = CommandSig::default();
                $(push_param(&mut sig, $name::KIND, $name::OPTIONAL);)+
                sig
            }

            fn from_args(args: &[String]) -> Result<Self, String> {
                let mut args = args.iter().map(String::as_str);
                Ok(($($name::parse(args.next())?,)+))
            }
        }
    };
}

from_args_tuple!(A);
from_args_tuple!(A, B);
from_args_tuple!(A, B, C);
from_args_tuple!(A, B, C, D);
from_args_tuple!(A, B, C, D, E);

#[derive(Default)]
pub struct Commands {
    handlers: HashMap<String, Handler>,
    signatures: BTreeMap<String, CommandSig>,
}

impl Commands {
    pub fn insert<A: FromArgs + 'static>(
        &mut self,
        name: impl Into<String>,
        handler: impl Fn(&mut GameContext, A) -> Option<ScreenState> + 'static,
    ) {
        let name = name.into();
        let signature = A::signature();
        let checked = signature.clone();
        let label = name.clone();

        self.handlers.insert(
            name.clone(),
            Box::new(move |ctx, args| {
                if let Err(e) = checked.check(&label, args) {
                    eprintln!("⚠️ {}", e);
                    return None;
                }
                match A::from_args(args) {
                    Ok(parsed) => handler(ctx, parsed),
                    Err(e) => {
                        eprintln!("⚠️ `call {}`: {}", label, e);
                        None
                    }
                }
            }),
        );
        self.signatures.insert(name, signature);
    }

    pub fn contains(&self, name: &str) -> bool {
        self.handlers.contains_key(name)
    }

    pub fn signatures(&self) -> &BTreeMap<String, CommandSig> {
        &self.signatures
    }

    pub fn run(&self, name: &str, ctx: &mut GameContext, args: &[String]) -> Option<ScreenState> {
        match self.handlers.get(name) {
            Some(handler) => handler(ctx, args),
            None => {
                eprintln!("⚠️ Unhandled command: call {} {}", name, args.join(" "));
                None
            }
        }
    }
}
