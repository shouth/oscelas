use crate::parser::decl::parse_qualified_behavior_name;
use crate::parser::member::{
    check_field_declaration, parse_constraint_declaration, parse_coverage_declaration,
    parse_event_declaration, parse_field_declaration, parse_method_declaration,
};
use crate::parser::{error_unexpected, Parser};
use crate::syntax::OscSyntaxKind::*;

pub fn parse_struct_declaration(p: &mut Parser) {
    let checkpoint = p.open();
    p.expect(STRUCT_KW);
    parse_qualified_behavior_name(p);

    if p.check(INHERITS_KW) {
        parse_struct_inherits_clause(p);
    }

    if p.check(COLON) {
        parse_struct_body(p);
    } else if p.eat(NEWLINE) {
        // new line
    } else {
        error_unexpected(p);
    }

    p.close(checkpoint, STRUCT_DECLARATION);
}

fn parse_struct_inherits_clause(p: &mut Parser) {
    let checkpoint = p.open();
    p.expect(INHERITS_KW);
    parse_qualified_behavior_name(p);

    let condition_checkpoint = p.open();
    if p.eat(LEFT_PAREN) {
        parse_qualified_behavior_name(p);
        p.expect(RIGHT_PAREN);
        p.close(condition_checkpoint, STRUCT_INHERITS_CONDITION);
    }
    p.close(checkpoint, STRUCT_INHERITS_CLAUSE);
}

fn parse_struct_body(p: &mut Parser) {
    let checkpoint = p.open();
    p.expect(COLON);
    p.expect(NEWLINE);
    p.expect(INDENT);
    parse_struct_member_list(p);
    p.expect(DEDENT);
    p.close(checkpoint, STRUCT_BODY);
}

fn parse_struct_member_list(p: &mut Parser) {
    let checkpoint = p.open();
    while !p.check(DEDENT | EOF) {
        if p.check(EVENT_KW) {
            parse_event_declaration(p);
        } else if p.check(KEEP_KW | REMOVE_DEFAULT_KW) {
            parse_constraint_declaration(p);
        } else if p.check(DEF_KW) {
            parse_method_declaration(p);
        } else if p.check(COVER_KW | RECORD_KW) {
            parse_coverage_declaration(p);
        } else if check_field_declaration(p) {
            parse_field_declaration(p);
        } else {
            error_unexpected(p);
            p.error();
        }
    }
    p.close(checkpoint, STRUCT_MEMBER_LIST);
}
