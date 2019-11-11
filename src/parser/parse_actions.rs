use crate::parser::{
    ast::*,
    parse_comments::comment,
    parse_ident::{parse_ident, parse_ident_no_check, parse_string, get_tag},
    parse_import::parse_import,
    parse_var_types::{parse_as_variable, parse_expr_list, parse_var_expr},
    tokens::*,
    tools::get_interval,
    GotoType,
};
use nom::{
    branch::alt, bytes::complete::tag, combinator::complete, error::ParseError, sequence::preceded,
    *,
};

pub fn parse_assignation<'a, E: ParseError<Span<'a>>>(s: Span<'a>) -> IResult<Span<'a>, Expr, E> {
    let (s, name) = parse_ident_no_check(s)?;
    let (s, _) = preceded(comment, tag(ASSIGN))(s)?;
    let (s, expr) = complete(alt((parse_as_variable, parse_var_expr)))(s)?;
    Ok((
        s,
        Expr::ObjectExpr(ObjectType::Assign(name, Box::new(expr))),
    ))
}

fn get_step<'a, E: ParseError<Span<'a>>>(s: Span<'a>) -> IResult<Span<'a>, GotoType, E> {
    let (s, ..) = get_tag(s, STEP)?;
    Ok((s, GotoType::Step))
}

fn get_hook<'a, E: ParseError<Span<'a>>>(s: Span<'a>) -> IResult<Span<'a>, GotoType, E> {
    let (s, ..) = preceded(comment, tag("@"))(s)?;
    Ok((s, GotoType::Hook))
}

fn get_flow<'a, E: ParseError<Span<'a>>>(s: Span<'a>) -> IResult<Span<'a>, GotoType, E> {
    let (s, ..) = get_tag(s, FLOW)?;
    Ok((s, GotoType::Flow))
}

fn get_default<'a, E: ParseError<Span<'a>>>(s: Span<'a>) -> IResult<Span<'a>, GotoType, E> {
    Ok((s, GotoType::Step))
}

fn parse_goto<'a, E: ParseError<Span<'a>>>(s: Span<'a>) -> IResult<Span<'a>, Expr, E> {
    let (s, ..) = get_tag(s, GOTO)?;
    let (s, goto_type) = alt((get_step, get_flow, get_hook, get_default))(s)?;
    let (s, name) = match parse_ident(s) {
        Ok(vars) => vars,
        Err(Err::Error(err)) | Err(Err::Failure(err)) => {
            return Err(Err::Error(E::add_context(
                s,
                "missing step name after goto",
                err,
            )))
        }
        Err(Err::Incomplete(needed)) => return Err(Err::Incomplete(needed)),
    };
    Ok((s, Expr::ObjectExpr(ObjectType::Goto(goto_type, name))))
}

fn parse_say<'a, E: ParseError<Span<'a>>>(s: Span<'a>) -> IResult<Span<'a>, Expr, E> {
    let (s, ..) = get_tag(s, SAY)?;
    let (s, expr) = complete(alt((parse_as_variable, parse_var_expr)))(s)?;
    Ok((s, Expr::ObjectExpr(ObjectType::Say(Box::new(expr)))))
}

fn parse_use<'a, E: ParseError<Span<'a>>>(s: Span<'a>) -> IResult<Span<'a>, Expr, E> {
    let (s, ..) = get_tag(s, USE)?;
    let (s, expr) = complete(alt((parse_as_variable, parse_var_expr)))(s)?;
    Ok((s, Expr::ObjectExpr(ObjectType::Use(Box::new(expr)))))
}

fn parse_hold<'a, E: ParseError<Span<'a> >>(s: Span<'a>) -> IResult<Span<'a>, Expr, E> {
    let (s, inter) = get_interval(s)?; 
    let (s, ..) = get_tag(s, HOLD)?;
    Ok((s, Expr::ObjectExpr(ObjectType::Hold(inter))))
}

fn parse_remember<'a, E: ParseError<Span<'a>>>(s: Span<'a>) -> IResult<Span<'a>, Expr, E> {
    let (s, ..) = get_tag(s, REMEMBER)?;
    let (s, expr) = parse_var_expr(s)?;

    let (s, _) = match get_tag(s, AS) {
        Ok(vars) => vars,
        Err(Err::Error(err)) | Err(Err::Failure(err)) => {
            return Err(Err::Error(E::add_context(
                s,
                "missing as name after remember var",
                err,
            )))
        }
        Err(Err::Incomplete(needed)) => return Err(Err::Incomplete(needed)),
    };
    let (s, ident) = preceded(comment, complete(parse_ident))(s)?;
    Ok((
        s,
        Expr::ObjectExpr(ObjectType::Remember(ident, Box::new(expr))),
    ))
}

pub fn parse_actions<'a, E: ParseError<Span<'a>>>(s: Span<'a>) -> IResult<Span<'a>, Expr, E> {
    // let (s, name) = parse_ident(s)?;
    let (s, name) = parse_ident_no_check(s)?;
    let (s, expr) = parse_expr_list(s)?;
    Ok((
        s,
        Expr::ObjectExpr(ObjectType::Normal(name, Box::new(expr))),
    ))
}

pub fn parse_hook<'a, E: ParseError<Span<'a> >>(s: Span<'a>) -> IResult<Span<'a>, Expr, E> {
    let (s, ..) = preceded(comment, tag("@"))(s)?;
    //TODO: add error if ident not found
    let (s, name) = parse_string(s)?;

    Ok((s, Expr::Hook(name)))
}

pub fn parse_root_functions<'a, E: ParseError<Span<'a>>>(
    s: Span<'a>,
) -> IResult<Span<'a>, Expr, E> {
    alt((
        parse_say,
        parse_remember,
        parse_import,
        parse_goto,
        parse_use,
        parse_hold,
    ))(s)
}
