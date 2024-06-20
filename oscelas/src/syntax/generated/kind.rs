#![allow(bad_style, missing_docs, unreachable_pub)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum OscDslSyntaxKind {
    EOF,
    DOT,
    DOT_DOT,
    COMMA,
    COLON,
    COLON_COLON,
    ASSIGN,
    AT,
    ARROW,
    LEFT_PAREN,
    RIGHT_PAREN,
    LEFT_BRACKET,
    RIGHT_BRACKET,
    QUESTION,
    EXCLAMATION,
    FAT_ARROW,
    EQUAL,
    NOT_EQUAL,
    LESS,
    LESS_EQUAL,
    GREATER,
    GREATER_EQUAL,
    PLUS,
    MINUS,
    STAR,
    SLASH,
    PERCENT,
    A_KW,
    ACTION_KW,
    ACTOR_KW,
    AND_KW,
    AS_KW,
    BOOL_KW,
    CALL_KW,
    CD_KW,
    COVER_KW,
    DEF_KW,
    DEFAULT_KW,
    DO_KW,
    ELAPSED_KW,
    EMIT_KW,
    ENUM_KW,
    EVENT_KW,
    EVERY_KW,
    EXPORT_KW,
    EXPRESSION_KW,
    EXTEND_KW,
    EXTERNAL_KW,
    FACTOR_KW,
    FALL_KW,
    FALSE_KW,
    FLOAT_KW,
    GLOBAL_KW,
    HARD_KW,
    IF_KW,
    IMPORT_KW,
    IN_KW,
    INF_KW,
    INHERITS_KW,
    INT_KW,
    IS_KW,
    IT_KW,
    K_KW,
    KEEP_KW,
    KG_KW,
    LIST_KW,
    M_KW,
    MODIFIER_KW,
    MOL_KW,
    NAMESPACE_KW,
    NAN_KW,
    NOT_KW,
    NULL_KW,
    OF_KW,
    OFFSET_KW,
    ON_KW,
    ONE_OF_KW,
    ONLY_KW,
    OR_KW,
    PARALLEL_KW,
    RAD_KW,
    RANGE_KW,
    RECORD_KW,
    REMOVE_DEFAULT_KW,
    RISE_KW,
    S_KW,
    SAMPLE_KW,
    SCENARIO_KW,
    SERIAL_KW,
    SI_KW,
    STRING_KW,
    STRUCT_KW,
    TRUE_KW,
    TYPE_KW,
    UINT_KW,
    UNDEFINED_KW,
    UNIT_KW,
    UNTIL_KW,
    USE_KW,
    VAR_KW,
    WAIT_KW,
    WITH_KW,
    INTEGER_LITERAL,
    FLOAT_LITERAL,
    STRING_LITERAL,
    NEWLINE,
    INDENT,
    DEDENT,
    IDENTIFIER,
    ERROR,
    WHITESPACE,
    COMMENT,
    TRIVIAL_NEWLINE,
    QUALIFIED_IDENTIFIER,
    PHYSICAL_LITERAL,
    OSC_FILE,
    PRELUDE_STATEMENT_LIST,
    MAIN_STATEMENT_LIST,
    IMPORT_STATEMENT,
    STRUCTURED_IDENTIFIER,
    STRUCTURED_IDENTIFIER_ELEMENT,
    NAMESPACE_STATEMENT,
    EXPORT_STATEMENT,
    NAMESPACE_USE_CLAUSE,
    NAMESPACE_LIST,
    NAMESPACE_LIST_ELEMENT,
    EXPORT_SPECIFICATION_LIST,
    EXPORT_SPECIFICATION_LIST_ELEMENT,
    EXPORT_WILDCARD_SPECIFICATION,
    PHYSICAL_TYPE_DECLARATION,
    UNIT_DECLARATION,
    ENUM_DECLARATION,
    STRUCT_DECLARATION,
    ACTOR_DECLARATION,
    ACTION_DECLARATION,
    SCENARIO_DECLARATION,
    MODIFIER_DECLARATION,
    GLOBAL_PARAMETER_DECLARATION,
    LIST_TYPE_DECLARATOR,
    QUALIFIED_BEHAVIOR_NAME,
    SI_BASE_UNIT_SPECIFIER,
    SI_UNIT_SPECIFIER,
    SI_BASE_EXPONENT_LIST,
    SI_BASE_EXPONENT_LIST_ELEMENT,
    SI_UNIT_SPECIFIER_ARGUMENT_LIST,
    SI_UNIT_SPECIFIER_ARGUMENT_LIST_ELEMENT,
    ENUM_MEMBER_DECL_LIST,
    ENUM_MEMBER_DECL,
    ENUM_INITIALIZER_CLAUSE,
    ENUM_VALUE_REFERENCE,
    STRUCT_INHERITS_CLAUSE,
    STRUCT_INHERITS_CONDITION,
    STRUCT_BODY,
    STRUCT_MEMBER_DECL_LIST,
    EVENT_DECLARATION,
    METHOD_DECLARATION,
    COVERAGE_DECLARATION,
    ACTOR_INHERITS_CLAUSE,
    ACTOR_INHERITS_CONDITION,
    ACTOR_BODY,
    ACTOR_MEMBER_DECL_LIST,
    SCENARIO_INHERITS_CLAUSE,
    SCENARIO_INHERITS_CONDITION,
    SCENARIO_BODY,
    SCENARIO_MEMBER_ITEM_LIST,
    MODIFIER_APPLICATION,
    ACTION_INHERITS_CLAUSE,
    ACTION_BODY,
    ACTION_INHERITS_CONDITION,
    ACTION_MEMBER_ITEM_LIST,
    MODIFIER_OF_CLAUSE,
    MODIFIER_BODY,
    MODIFIER_MEMBER_ITEM_LIST,
    ON_DIRECTIVE,
    ENUM_TYPE_EXTENSION,
    STRUCTURED_TYPE_EXTENSION,
    EXTENSION_MEMBER_DECL_LIST,
    PARAMETER_DECLARATION,
    VARIABLE_DECLARATION,
    KEEP_CONSTRAINT_DECLARATION,
    REMOVE_DEFAULT_DECLARATION,
    EVENT_ARGUMENT_LIST_SPECIFICATION,
    EVENT_IS_CLAUSE,
    ARGUMENT_LIST_SPECIFICATION,
    EVENT_REFERENCE_SPECIFICATION,
    EVENT_REFERENCE,
    EVENT_REFERENCE_CONDITION,
    EVENT_FIELD_DECL,
    MEMBER_REFERENCE,
    RISE_EXPRESSION,
    FALL_EXPRESSION,
    ELAPSED_EXPRESSION,
    EVERY_EXPRESSION,
    EVERY_EXP_OFFSET,
    FIELD_NAME_LIST,
    PARAMETER_INITIALIZER_CLAUSE,
    FIELD_NAME_LIST_ELEMENT,
    VARIABLE_INITIALIZER_CLAUSE,
    SAMPLE_EXPRESSION,
    PARAMETER_WITH_DECLARATION,
    METHOD_RETURN_TYPE,
    METHOD_IMPLEMENTATION,
    METHOD_EXPRESSION_BODY,
    METHOD_EXTERNAL_BODY,
    ARGUMENT_LIST,
    DO_DIRECTIVE,
    ON_MEMBER_LIST,
    CALL_DIRECTIVE,
    EMIT_DIRECTIVE,
    DO_MEMBER,
    COMPOSITION,
    BEHAVIOR_INVOCATION,
    WAIT_DIRECTIVE,
    COMPOSITION_ARGUMENTS,
    DO_MEMBER_LIST,
    BEHAVIOR_WITH_DECLARATION,
    BEHAVIOR_WITH_MEMBER_LIST,
    UNTIL_DIRECTIVE,
    EMIT_ARGUMENTS,
    METHOD_INVOCATION,
    ARGUMENT_LIST_SPECIFICATION_ELEMENT,
    ARGUMENT_SPECIFICATION,
    ARGUMENT_INITIALIZER_CLAUSE,
    LABELED_ARGUMENT,
    TERNARY_OP_EXP,
    LOGICAL_OP_EXP,
    BINARY_OP_EXP,
    UNARY_OP_EXP,
    CAST_EXP,
    TYPE_TEST_EXP,
    ELEMENT_ACCESS,
    FUNCTION_APPLICATION,
    PARENTHESIZED_EXP,
    LIST_CONSTRUCTOR,
    EXPRESSION_LIST,
    EXPRESSION_LIST_ELEMENT,
    PARENTHESES_RANGE_CONSTRUCTOR,
    BRACKETS_RANGE_CONSTRUCTOR,
    #[doc(hidden)]
    __LAST,
}