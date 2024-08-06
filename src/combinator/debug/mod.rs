#![cfg_attr(feature = "debug", allow(clippy::std_instead_of_core))]

#[cfg(feature = "debug")]
mod internals;

use crate::error::ErrMode;
use crate::stream::Stream;
use crate::Parser;

/// Trace the execution of the parser
///
/// Note that [`Parser::context`] also provides high level trace information.
///
/// See [tutorial][crate::_tutorial::chapter_8] for more details.
///
/// # Example
///
/// ```rust
/// # use winnow::{error::ErrMode, error::{InputError, ErrorKind}, error::Needed};
/// # use winnow::token::take_while;
/// # use winnow::stream::AsChar;
/// # use winnow::prelude::*;
/// use winnow::combinator::trace;
///
/// fn short_alpha<'s>(s: &mut &'s [u8]) -> PResult<&'s [u8], InputError<&'s [u8]>> {
///   trace("short_alpha",
///     take_while(3..=6, AsChar::is_alpha)
///   ).parse_next(s)
/// }
///
/// assert_eq!(short_alpha.parse_peek(b"latin123"), Ok((&b"123"[..], &b"latin"[..])));
/// assert_eq!(short_alpha.parse_peek(b"lengthy"), Ok((&b"y"[..], &b"length"[..])));
/// assert_eq!(short_alpha.parse_peek(b"latin"), Ok((&b""[..], &b"latin"[..])));
/// assert_eq!(short_alpha.parse_peek(b"ed"), Err(ErrMode::Backtrack(InputError::new(&b"ed"[..], ErrorKind::Slice))));
/// assert_eq!(short_alpha.parse_peek(b"12345"), Err(ErrMode::Backtrack(InputError::new(&b"12345"[..], ErrorKind::Slice))));
/// ```
#[cfg_attr(not(feature = "debug"), allow(unused_variables))]
#[cfg_attr(not(feature = "debug"), allow(unused_mut))]
#[cfg_attr(not(feature = "debug"), inline(always))]
pub fn trace<I: Stream, O, E>(
    name: impl crate::lib::std::fmt::Display,
    parser: impl Parser<I, O, E>,
) -> impl Parser<I, O, E> {
    #[cfg(all(feature = "debug", debug_assertions))]
    {
        if let Some(flag) = option_env!("WINNOW_DEBUG") {
            return internals::Trace::new(parser, name, flag.parse::<bool>().unwrap_or_default());
        }
        return internals::Trace::new(parser, name, false);
    }
    #[cfg(any(not(feature = "debug"), not(debug_assertions)))]
    TraceEmpty::new(parser, name)
}

#[cfg_attr(not(feature = "debug"), allow(unused_variables))]
pub(crate) fn trace_result<T, E>(
    name: impl crate::lib::std::fmt::Display,
    res: &Result<T, ErrMode<E>>,
) {
    #[cfg(all(feature = "debug", debug_assertions))]
    {
        if let Some(flag) = option_env!("WINNOW_DEBUG") {
            if flag.parse::<bool>().unwrap_or_default() {
                let depth = internals::Depth::existing();
                let severity = internals::Severity::with_result(res);
                internals::result(*depth, &name, severity);
            }
        }
    }
}

pub(crate) struct DisplayDebug<D>(pub(crate) D);

impl<D: crate::lib::std::fmt::Debug> crate::lib::std::fmt::Display for DisplayDebug<D> {
    fn fmt(&self, f: &mut crate::lib::std::fmt::Formatter<'_>) -> crate::lib::std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[test]
#[cfg(feature = "std")]
#[cfg_attr(miri, ignore)]
#[cfg(unix)]
#[cfg(feature = "debug")]
fn example() {
    use term_transcript::{test::TestConfig, ShellOptions};

    let path = snapbox::cmd::compile_example("string", ["--features=debug"]).unwrap();

    let current_dir = path.parent().unwrap();
    let cmd = path.file_name().unwrap();
    // HACK: term_transcript doesn't allow non-UTF8 paths
    let cmd = format!("./{}", cmd.to_string_lossy());

    TestConfig::new(
        ShellOptions::default()
            .with_current_dir(current_dir)
            .with_env("CLICOLOR_FORCE", "1"),
    )
    .test("assets/trace.svg", [cmd.as_str()]);
}

pub(crate) struct TraceEmpty<P, D, I, O, E>
where
    P: Parser<I, O, E>,
    I: Stream,
    D: std::fmt::Display,
{
    parser: P,
    _name: D,
    i: core::marker::PhantomData<I>,
    o: core::marker::PhantomData<O>,
    e: core::marker::PhantomData<E>,
}

impl<P, D, I, O, E> TraceEmpty<P, D, I, O, E>
where
    P: Parser<I, O, E>,
    I: Stream,
    D: std::fmt::Display,
{
    #[inline(always)]
    pub(crate) fn new(parser: P, name: D) -> Self {
        Self {
            parser,
            _name: name,
            i: Default::default(),
            o: Default::default(),
            e: Default::default(),
        }
    }
}

impl<P, D, I, O, E> Parser<I, O, E> for TraceEmpty<P, D, I, O, E>
where
    P: Parser<I, O, E>,
    I: Stream,
    D: std::fmt::Display,
{
    #[inline]
    fn parse_next(&mut self, i: &mut I) -> crate::PResult<O, E> {
        self.parser.parse_next(i)
    }
}
