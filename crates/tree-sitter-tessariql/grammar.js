// TessariQL grammar for tree-sitter — error-tolerant, resilient by
// construction (tree-sitter always produces a tree). Authored against the
// engine-validated corpus in ../../corpus/statements (172 statements,
// tessaridb 0.50.0); keywords are uppercase per the language's convention,
// identifiers stay identifiers everywhere else.
//
// Statements are `;`-terminated (as in the language itself): after a
// statement the parser always expects `;`, so clause boundaries are LR-clean
// and the table stays small. Expressions are leveled (one rule per
// precedence level), the LR-friendly shape.
//
// Generate with: npx -y tree-sitter-cli@0.25.10 generate --abi 14

const kw = (s) => s;

module.exports = grammar({
  name: "tessariql",

  word: ($) => $.identifier,

  extras: ($) => [/\s/, /\/\/[^\n]*/, /#[^\n]*/],

  supertypes: ($) => [$.statement, $.expression],

  rules: {
    source_file: ($) => repeat(seq($.statement, ";")),

    identifier: ($) => /[a-zA-Z_][a-zA-Z0-9_]*/,
    number: ($) => /[0-9]+(\.[0-9]+)?/,
    string: ($) => /'([^'\\]|\\.)*'/,
    duration: ($) => /[0-9]+(\.[0-9]+)?(ns|us|ms|s|m|h|d)/,
    parameter: ($) => /\$[a-zA-Z_][a-zA-Z0-9_]*/,

    // ------------------------------------------------------------------
    // statements
    // ------------------------------------------------------------------

    statement: ($) =>
      choice(
        $.use_statement,
        $.define_namespace_statement,
        $.define_database_statement,
        $.define_table_statement,
        $.define_collection_statement,
        $.define_field_statement,
        $.define_index_statement,
        $.define_analyzer_statement,
        $.define_event_statement,
        $.define_graph_statement,
        $.define_edge_statement,
        $.define_geo_statement,
        $.define_vector_statement,
        $.define_queue_statement,
        $.define_topic_statement,
        $.define_series_statement,
        $.define_view_statement,
        $.define_user_statement,
        $.define_vault_statement,
        $.define_space_statement,
        $.define_param_statement,
        $.define_node_statement,
        $.define_replica_statement,
        $.define_replication_statement,
        $.define_failover_statement,
        $.define_ai_provider_statement,
        $.alter_user_statement,
        $.alter_table_statement,
        $.alter_replication_statement,
        $.drop_statement,
        $.explain_statement,
        $.select_statement,
        $.create_statement,
        $.insert_statement,
        $.update_statement,
        $.upsert_statement,
        $.delete_statement,
        $.relate_statement,
        $.read_from_statement,
        $.info_statement,
        $.begin_statement,
        $.commit_statement,
        $.cancel_statement,
        $.verify_statement,
        $.savepoint_statement,
        $.rollback_statement,
        $.let_statement,
        $.return_statement,
        $.throw_statement,
        $.grant_statement,
        $.revoke_statement,
        $.backup_statement,
        $.check_table_statement,
        $.analyze_table_statement,
        $.rebuild_index_statement,
        $.claim_statement,
        $.release_statement,
        $.unseal_vault_statement,
        $.seal_vault_statement,
        $.change_vault_passphrase_statement,
        $.reveal_statement
      ),

    if_not_exists: ($) => seq(kw("IF"), kw("NOT"), kw("EXISTS")),

    use_statement: ($) => seq(kw("USE"), choice(kw("NAMESPACE"), kw("DATABASE")), $.identifier),

    define_namespace_statement: ($) =>
      seq(kw("DEFINE"), kw("NAMESPACE"), optional($.if_not_exists), $.identifier),

    define_database_statement: ($) =>
      seq(kw("DEFINE"), kw("DATABASE"), optional($.if_not_exists), $.identifier),

    type_name: ($) =>
      choice(
        kw("string"), kw("int"), kw("float"), kw("bool"), kw("datetime"),
        kw("duration"), kw("decimal"), kw("number"), kw("object"), kw("array"),
        kw("set"), kw("record"), kw("geometry"), kw("uuid"), kw("bytes"),
        kw("regex"), kw("null"), kw("none"), kw("json"), kw("jsonb"),
        seq(kw("record"), "(", commaSep1($.identifier), ")"),
        seq(kw("vector"), "<", $.number, ">"),
        seq(kw("string"), $.string)
      ),

    column_definition: ($) => seq($.identifier, $.type_name),

    table_clause: ($) =>
      choice(
        kw("SCHEMALESS"),
        kw("SCHEMAFULL"),
        seq(kw("IN"), $.identifier),
        seq(kw("IDENTITY"), choice(kw("UUID"), kw("COUNTER"), $.identifier)),
        seq(kw("STORE"), kw("COLUMN"), optional(seq(kw("PARTITIONS"), $.number, kw("BY"), $.identifier)), optional(seq(kw("SORT"), kw("BY"), $.identifier))),
        seq(kw("SPLIT"), kw("AT"), commaSep1($.string)),
        seq(kw("SPLIT"), kw("AUTOMATICALLY")),
        seq(kw("EXPIRE"), optional(seq(kw("AFTER"), $.duration))),
        seq(kw("REFERENCES"), $.identifier, optional(seq(kw("ON"), kw("DELETE"), $.identifier)))
      ),

    define_table_statement: ($) =>
      seq(
        kw("DEFINE"), kw("TABLE"), optional($.if_not_exists), $.identifier,
        optional(seq("(", commaSep1($.column_definition), ")")),
        repeat($.table_clause)
      ),

    define_collection_statement: ($) =>
      seq(kw("DEFINE"), kw("COLLECTION"), optional($.if_not_exists), $.identifier,
        optional(seq(kw("IN"), $.identifier))),

    field_path: ($) => seq($.identifier, repeat(choice(seq(".", $.identifier), seq("[", $.expression, "]")))),

    field_clause: ($) =>
      choice(
        seq(kw("TYPE"), $.type_name),
        kw("REQUIRED"),
        kw("OPTIONAL"),
        seq(kw("DEFAULT"), $.expression),
        seq(kw("ASSERT"), $.expression),
        seq(kw("ANALYZER"), $.identifier),
        seq(kw("REFERENCES"), $.identifier,
          optional(seq(kw("ON"), kw("DELETE"), choice(kw("CASCADE"), kw("SET"), kw("NONE"), kw("REJECT"), $.identifier))))
      ),

    define_field_statement: ($) =>
      seq(
        kw("DEFINE"), kw("FIELD"), optional($.if_not_exists), $.field_path,
        kw("ON"), $.identifier,
        repeat($.field_clause)
      ),

    define_index_statement: ($) =>
      seq(
        kw("DEFINE"), kw("INDEX"), optional($.if_not_exists), $.identifier,
        kw("ON"), $.identifier, kw("FIELDS"), commaSep1($.field_path),
        optional(choice(
          kw("UNIQUE"),
          kw("CONTAINS"),
          seq(kw("SEARCH"), optional(seq(kw("ANALYZER"), $.identifier)),
              optional(choice(seq(kw("POSITIONS"), optional(kw("OFFSETS"))), kw("NO"), kw("SCORE")))),
          kw("SPATIAL")
        ))
      ),

    define_analyzer_statement: ($) =>
      seq(kw("DEFINE"), kw("ANALYZER"), optional($.if_not_exists), $.identifier,
        kw("FILTERS"), commaSep1($.identifier)),

    define_event_statement: ($) =>
      seq(
        kw("DEFINE"), kw("EVENT"), $.identifier, kw("ON"), $.identifier,
        optional(seq(kw("FOR"), commaSep1(choice(kw("CREATE"), kw("UPDATE"), kw("DELETE"))))),
        optional(seq(kw("WHEN"), $.expression)),
        kw("THEN"), choice($.block, $.statement)
      ),

    block: ($) => seq("{", repeat(seq($.statement, ";")), "}"),

    define_graph_statement: ($) => seq(kw("DEFINE"), kw("GRAPH"), $.identifier),

    define_edge_statement: ($) =>
      seq(kw("DEFINE"), kw("EDGE"), $.identifier, kw("IN"), $.identifier,
        kw("FROM"), $.identifier, kw("TO"), $.identifier),

    define_geo_statement: ($) => seq(kw("DEFINE"), kw("GEO"), $.identifier),

    define_vector_statement: ($) =>
      seq(kw("DEFINE"), kw("VECTOR"), $.identifier,
        kw("DIMENSION"), $.number, kw("DISTANCE"), $.identifier),

    define_queue_statement: ($) =>
      seq(
        kw("DEFINE"), kw("QUEUE"), $.identifier,
        kw("TIMEOUT"), $.duration,
        optional(seq(kw("ATTEMPTS"), $.number)),
        optional(seq(kw("NOT"), kw("BEFORE"), $.identifier)),
        optional(seq(kw("DEAD"), kw("LETTER"), kw("TO"), $.identifier))
      ),

    define_topic_statement: ($) =>
      seq(
        kw("DEFINE"), kw("TOPIC"), $.identifier,
        repeat(choice(
          seq(kw("RETAIN"), choice($.duration, seq(kw("BYTES"), $.number))),
          seq(kw("DEDUPLICATE"), $.duration, kw("ON"), $.identifier),
          seq(kw("PARTITIONS"), $.number, kw("BY"), $.identifier),
          seq(kw("IN"), kw("MEMORY"), optional(seq(kw("MAX"), $.number)))
        ))
      ),

    define_series_statement: ($) =>
      seq(kw("DEFINE"), kw("SERIES"), $.identifier,
        kw("RETAIN"), $.duration, kw("TIME"), $.identifier,
        optional(seq(kw("ROLLUP"), $.duration))),

    define_view_statement: ($) =>
      seq(kw("DEFINE"), kw("VIEW"), $.identifier,
        optional(choice(kw("MATERIALIZED"), seq(kw("AS"), kw("MATERIALIZED")))),
        kw("AS"), $.select_statement),

    user_reach: ($) =>
      choice(
        seq(kw("NAMESPACE"), $.identifier),
        kw("DATABASE"),
        seq($.identifier, ".", $.identifier),
        $.identifier
      ),

    define_user_statement: ($) =>
      seq(
        kw("DEFINE"), kw("USER"), optional($.if_not_exists), $.identifier,
        optional(seq(kw("ON"), $.user_reach)),
        choice(
          seq(kw("ROLE"), $.identifier),
          seq(kw("AUTHORITIES"), commaSep1($.identifier))
        ),
        choice(
          seq(kw("PASSWORD"), $.string),
          seq(kw("PASSHASH"), $.string)
        )
      ),

    define_vault_statement: ($) =>
      seq(kw("DEFINE"), kw("VAULT"), $.identifier, optional(seq(kw("PASSPHRASE"), $.string))),

    define_space_statement: ($) =>
      seq(
        kw("DEFINE"), kw("SPACE"), $.identifier,
        optional(seq(kw("IN"), kw("MEMORY"), optional(seq(kw("MAX"), $.number)))),
        optional(seq(kw("MAX"), $.number)),
        optional(seq(kw("EVICT"), kw("NONE")))
      ),

    define_param_statement: ($) =>
      seq(kw("DEFINE"), kw("PARAM"), optional(seq(kw("OR"), kw("REPLACE"))),
        $.parameter, kw("VALUE"), $.expression),

    define_node_statement: ($) =>
      seq(
        kw("DEFINE"), kw("NODE"),
        choice(
          seq(kw("ROLES"), commaSep1($.identifier), optional(seq(kw("ENDPOINTS"), $.string))),
          seq(kw("ENDPOINTS"), $.string)
        )
      ),

    define_replica_statement: ($) =>
      seq(
        kw("DEFINE"), kw("REPLICA"), $.identifier, kw("AT"), $.string,
        optional(seq(kw("ROLES"), commaSep1($.identifier))),
        repeat(seq(kw("REPLICATES"),
          choice(
            kw("STORE"),
            seq(kw("NAMESPACE"), $.identifier),
            seq(kw("DATABASE"), $.identifier),
            seq(kw("TABLE"), $.identifier)
          )))
      ),

    define_replication_statement: ($) =>
      seq(
        kw("DEFINE"), kw("REPLICATION"), $.identifier,
        choice(kw("TO"), kw("FROM")), $.string,
        kw("FOR"), kw("DATABASE"), seq($.identifier, ".", $.identifier),
        optional(seq(kw("AS"), $.string)),
        optional(seq(kw("SEED"), kw("AT"), $.number))
      ),

    define_failover_statement: ($) =>
      seq(
        kw("DEFINE"), kw("FAILOVER"),
        kw("AWARENESS"), $.duration,
        kw("COLLECTION"), $.duration,
        kw("ROUND"), $.duration,
        kw("CAMPAIGN"), $.duration,
        kw("LEASE"), $.duration,
        optional(seq(kw("BALANCE"), kw("LEADERSHIPS")))
      ),

    // Every clause is required and none has a default — the engine's own
    // rule for a value the store would otherwise guess.
    define_ai_provider_statement: ($) =>
      seq(
        kw("DEFINE"), kw("AI"), kw("PROVIDER"), optional($.if_not_exists), $.identifier,
        kw("URL"), $.string,
        kw("MODEL"), $.string,
        kw("KEY"), kw("VAULT"), $.identifier,
        kw("FIELD"), $.identifier,
        kw("BUDGET"), $.number, kw("PER"), $.duration,
        kw("TIMEOUT"), $.duration
      ),

    alter_user_statement: ($) =>
      seq(kw("ALTER"), kw("USER"), $.identifier, kw("SET"),
        choice(
          seq(kw("PASSWORD"), $.string),
          seq(kw("ROLE"), $.identifier),
          kw("LOCKED"),
          kw("UNLOCKED")
        )),

    alter_table_statement: ($) =>
      seq(
        kw("ALTER"), kw("TABLE"), $.identifier,
        choice(
          seq(kw("ADD"), kw("FIELD"), $.identifier, kw("TYPE"), $.type_name),
          seq(kw("DROP"), kw("FIELD"), $.identifier),
          seq(kw("RENAME"), kw("FIELD"), $.identifier, kw("TO"), $.identifier),
          seq(kw("RENAME"), kw("TO"), $.identifier),
          seq(kw("SPLIT"), kw("AT"), commaSep1($.string)),
          seq(kw("MERGE"), kw("SHARD"), commaSep1($.number)),
          seq(kw("SET"), choice(kw("SCHEMAFULL"), kw("SCHEMALESS")))
        )
      ),

    alter_replication_statement: ($) =>
      seq(kw("ALTER"), kw("REPLICATION"), $.identifier,
        choice(kw("PAUSED"), kw("RESUMED"), kw("ACTIVE"))),

    drop_statement: ($) =>
      seq(
        kw("DROP"),
        choice(
          seq(kw("TABLE"), $.identifier),
          seq(kw("COLLECTION"), $.identifier),
          seq(kw("INDEX"), optional(seq(kw("IF"), optional(kw("NOT")), kw("EXISTS"))), $.identifier, optional(seq(kw("ON"), $.identifier))),
          seq(kw("EVENT"), $.identifier, kw("ON"), $.identifier),
          seq(kw("PARAM"), $.parameter),
          seq(kw("VIEW"), $.identifier),
          seq(kw("GRAPH"), $.identifier),
          seq(kw("AI"), kw("PROVIDER"), $.identifier),
          seq(kw("ANALYZER"), $.identifier)
        ),
        optional(seq(kw("IF"), kw("EXISTS")))
      ),

    // ------------------------------------------------------------------
    // SELECT
    // ------------------------------------------------------------------

    explain_statement: ($) => seq(kw("EXPLAIN"), $.select_statement),

    select_clause: ($) =>
      choice(
        kw("APPROXIMATE"),
        seq(kw("USING"), kw("INDEX"), $.identifier),
        seq(kw("LATEST"), kw("BY"), commaSep1($.identifier)),
        prec(2, seq(kw("FILL"), choice(kw("NULL"), kw("PREVIOUS"), kw("LINEAR"), $.expression),
          kw("FROM"), choice(seq(kw("datetime"), $.string), $.expression),
          kw("TO"), choice(seq(kw("datetime"), $.string), $.expression))),
        seq(kw("OMIT"), commaSep1($.field_path)),
        seq(kw("FROM"), commaSep1($.source), repeat($.join_clause)),
        $.with_clause,
        seq(kw("WHERE"), $.expression),
        seq(kw("GROUP"), kw("BY"), commaSep1($.expression)),
        seq(kw("HAVING"), $.expression),
        $.order_clause,
        seq(kw("AFTER"), $.expression),
        seq(kw("START"), $.number),
        $.limit_clause,
        $.timeout_clause,
        $.version_clause,
        $.consistency_clause
      ),

    select_statement: ($) =>
      seq(
        kw("SELECT"),
        optional(kw("DISTINCT")),
        commaSep1($.projection_item),
        repeat($.select_clause)
      ),

    projection_item: ($) =>
      choice(
        "*",
        seq($.expression, optional(seq(kw("AS"), $.identifier)))
      ),

    record_ref: ($) =>
      seq($.identifier, ":", choice($.number, $.string, $.parameter, $.identifier)),

    traversal: ($) =>
      seq(
        choice($.record_ref, $.identifier),
        repeat1(seq("->", $.identifier)),
        optional(seq(kw("DEPTH"), $.number))
      ),

    relate_path: ($) =>
      seq(
        choice($.record_ref, $.identifier),
        repeat1(seq("->", choice($.record_ref, $.identifier)))
      ),

    source: ($) =>
      seq(
        optional(kw("ONLY")),
        choice($.traversal, $.record_ref, $.identifier, $.parenthesized_subquery),
        optional(seq(optional(kw("AS")), $.identifier))
      ),

    parenthesized_subquery: ($) => prec(1, seq("(", $.select_statement, ")")),

    join_clause: ($) =>
      seq(
        choice(
          seq(optional(choice(kw("LEFT"), kw("RIGHT"), kw("FULL"))), kw("JOIN")),
          seq(kw("ASOF"), kw("JOIN"))
        ),
        $.source,
        kw("ON"), $.expression
      ),

    with_clause: ($) => seq(kw("WITH"), commaSep1(seq($.identifier, kw("AS"), $.expression))),

    order_clause: ($) =>
      seq(
        kw("ORDER"), kw("BY"),
        choice(
          commaSep1($.order_item),
          seq(kw("FUSE"), "(", commaSep1($.order_item), ")", optional(seq(kw("DEPTH"), $.number)))
        )
      ),

    order_item: ($) =>
      seq($.expression, optional(choice(kw("ASC"), kw("DESC"))),
        optional(seq(kw("NULLS"), choice(kw("FIRST"), kw("LAST"))))),

    limit_clause: ($) => seq(kw("LIMIT"), choice($.number, kw("ALL"), $.parameter), optional(seq(kw("OFFSET"), $.number))),

    timeout_clause: ($) => seq(kw("TIMEOUT"), $.duration),

    version_clause: ($) => seq(kw("VERSION"), $.number),

    consistency_clause: ($) =>
      choice(kw("LEADER"), kw("SNAPSHOT"), seq(kw("STALE"), kw("WITHIN"), $.duration)),

    // ------------------------------------------------------------------
    // writes
    // ------------------------------------------------------------------

    create_statement: ($) =>
      seq(kw("CREATE"), choice($.record_ref, $.identifier), choice(seq("=", $.expression), seq(kw("CONTENT"), $.expression), $.block)),

    insert_statement: ($) =>
      seq(kw("INSERT"), kw("INTO"), $.identifier,
        "(", commaSep1($.identifier), ")",
        kw("VALUES"), commaSep1(seq("(", commaSep1($.expression), ")"))),

    assignment: ($) => seq($.field_path, "=", $.expression),

    update_statement: ($) =>
      seq(
        kw("UPDATE"), choice($.record_ref, $.identifier),
        choice(
          seq(kw("SET"), commaSep1($.assignment)),
          seq(kw("MERGE"), $.expression),
          seq("=", $.expression)
        ),
        optional(seq(kw("WHERE"), $.expression)),
        optional($.limit_clause),
        optional($.return_clause)
      ),

    upsert_statement: ($) =>
      seq(kw("UPSERT"), choice($.record_ref, $.identifier), "=", $.expression, optional($.return_clause)),

    return_clause: ($) => seq(kw("RETURN"), choice(kw("BEFORE"), kw("AFTER"))),

    delete_statement: ($) =>
      seq(
        kw("DELETE"),
        choice(
          seq(kw("FROM"), $.identifier, kw("WHERE"), $.expression, optional($.limit_clause)),
          seq($.record_ref, optional($.return_clause))
        )
      ),

    relate_statement: ($) =>
      seq(kw("RELATE"), $.relate_path, optional(seq("=", $.expression))),

    read_from_statement: ($) =>
      seq(
        kw("READ"), kw("FROM"), $.identifier,
        optional(seq(kw("PARTITION"), $.number)),
        optional(seq(kw("AFTER"), $.number)),
        optional(seq(kw("SINCE"), choice($.duration, seq(kw("datetime"), $.string)))),
        optional(seq(kw("FOR"), kw("CONSUMER"), $.string)),
        optional($.limit_clause)
      ),

    // ------------------------------------------------------------------
    // transactions, control, misc
    // ------------------------------------------------------------------

    begin_statement: ($) => kw("BEGIN"),
    commit_statement: ($) => seq(kw("COMMIT"), optional(seq(kw("ACROSS"), kw("LEADERS")))),
    cancel_statement: ($) => kw("CANCEL"),
    verify_statement: ($) => kw("VERIFY"),
    savepoint_statement: ($) => seq(kw("SAVEPOINT"), $.identifier),
    rollback_statement: ($) => seq(kw("ROLLBACK"), kw("TO"), $.identifier),

    let_statement: ($) => seq(kw("LET"), $.parameter, "=", $.expression),
    return_statement: ($) => seq(kw("RETURN"), $.expression),
    throw_statement: ($) => seq(kw("THROW"), $.expression),

    grant_statement: ($) => seq(kw("GRANT"), commaSep1($.identifier), kw("ON"), $.grant_target, kw("TO"), $.identifier),
    revoke_statement: ($) => seq(kw("REVOKE"), commaSep1($.identifier), kw("ON"), $.grant_target, kw("FROM"), $.identifier),

    grant_target: ($) =>
      choice(
        seq(kw("DATABASE"), $.identifier),
        seq(kw("NAMESPACE"), $.identifier),
        kw("STORE"),
        seq($.identifier, ".", $.identifier),
        seq($.identifier, optional(seq(kw("FIELDS"), commaSep1($.identifier))))
      ),

    backup_statement: ($) =>
      seq(
        kw("BACKUP"),
        optional(choice(kw("STATE"), kw("LOG"), kw("SCRIPT"))),
        optional(seq(kw("FROM"), $.number)),
        optional(seq(kw("OF"),
          choice(
            seq(kw("NAMESPACE"), $.identifier),
            seq(kw("DATABASE"), $.identifier),
            seq($.identifier, ".", $.identifier)
          ))),
        optional(seq(kw("TO"), $.string))
      ),

    check_table_statement: ($) => seq(kw("CHECK"), kw("TABLE"), $.identifier),
    analyze_table_statement: ($) => seq(kw("ANALYZE"), kw("TABLE"), $.identifier),
    rebuild_index_statement: ($) => seq(kw("REBUILD"), kw("INDEX"), $.identifier, optional(seq(kw("ON"), $.identifier))),

    claim_statement: ($) =>
      seq(kw("CLAIM"), $.record_ref,
        optional(seq(kw("FOR"), kw("CONSUMER"), $.string)),
        optional(seq(kw("NOT"), kw("BEFORE"), $.duration))),

    release_statement: ($) =>
      seq(kw("RELEASE"), $.record_ref,
        optional(seq(kw("FOR"), kw("CONSUMER"), $.string)),
        optional(seq(kw("NOT"), kw("BEFORE"), $.duration))),

    unseal_vault_statement: ($) => seq(kw("UNSEAL"), kw("VAULT"), $.identifier, kw("WITH"), $.string),
    seal_vault_statement: ($) => seq(kw("SEAL"), kw("VAULT"), $.identifier),
    change_vault_passphrase_statement: ($) =>
      seq(kw("CHANGE"), kw("VAULT"), $.identifier, kw("PASSPHRASE"), kw("FROM"), $.string, kw("TO"), $.string),
    reveal_statement: ($) => seq(kw("REVEAL"), $.record_ref),

    info_statement: ($) =>
      seq(
        kw("INFO"), kw("FOR"),
        choice(
          kw("STORE"),
          seq(kw("NAMESPACE"), optional($.identifier)),
          seq(kw("DATABASE"), optional($.identifier)),
          seq(kw("TABLE"), $.identifier),
          seq(kw("GRAPH"), $.identifier),
          seq(kw("VECTOR"), $.identifier),
          seq(kw("GEO"), $.identifier),
          seq(kw("QUEUE"), $.identifier),
          seq(kw("TOPIC"), $.identifier),
          seq(kw("SERIES"), $.identifier),
          seq(kw("VIEW"), $.identifier),
          seq(kw("USER"), $.identifier),
          kw("USERS"),
          seq(kw("ACCESS"), optional(seq(kw("TO"), choice(
            seq(kw("TABLE"), $.identifier),
            seq(kw("NAMESPACE"), $.identifier),
            seq(kw("DATABASE"), $.identifier),
            $.identifier
          )))),
          kw("NODE"),
          seq(kw("SEAL"), optional(seq(kw("OF"), $.identifier))),
          seq(kw("HISTORY"), kw("OF"), $.record_ref),
          seq(kw("VERSIONS"), kw("OF"), $.record_ref),
          seq(kw("TOPIC"), kw("CONSUMER"), optional($.identifier)),
          seq(kw("KAFKA"), kw("CONSUMER"), optional($.identifier)),
          kw("TOPOLOGY"), kw("TOPOLOGIES"),
          seq(kw("REPLICATION"), optional($.identifier)),
          kw("REPLICATIONS")
        )
      ),

    // ------------------------------------------------------------------
    // expressions (leveled, one rule per precedence)
    // ------------------------------------------------------------------

    expression: ($) => $.expr_or,

    expr_or: ($) =>
      prec.left(1, seq($.expr_and, repeat(seq(kw("OR"), $.expr_and)))),

    expr_and: ($) =>
      prec.left(2, seq($.expr_coalesce, repeat(seq(kw("AND"), $.expr_coalesce)))),

    expr_coalesce: ($) =>
      prec.left(3, seq($.expr_compare, repeat(seq("??", $.expr_compare)))),

    expr_compare: ($) =>
      prec.left(4, seq($.expr_membership,
        repeat(seq(choice("=", "!=", "<>", "<", "<=", ">", ">=", kw("LIKE")), $.expr_membership)))),

    expr_membership: ($) =>
      prec.left(5, seq($.expr_additive,
        repeat(seq(choice(kw("IN"), kw("CONTAINS"), $.matches_op), $.expr_additive)))),

    expr_additive: ($) =>
      prec.left(6, seq($.expr_multiplicative, repeat(seq(choice("+", "-"), $.expr_multiplicative)))),

    expr_multiplicative: ($) =>
      prec.left(7, seq($.unary_expression, repeat(seq(choice("*", "/", "%"), $.unary_expression)))),

    matches_op: ($) => seq(kw("MATCHES"), optional(choice(kw("PREFIX"), kw("INFIX"), kw("FUZZY"), kw("OR")))),

    unary_expression: ($) =>
      choice(
        prec(8, seq(kw("NOT"), $.unary_expression)),
        prec(8, seq("-", $.unary_expression)),
        $.primary_expression
      ),

    primary_expression: ($) =>
      choice(
        $.number,
        $.string,
        $.duration,
        $.parameter,
        kw("NONE"),
        kw("NULL"),
        kw("TRUE"),
        kw("FALSE"),
        seq(kw("datetime"), $.string),
        seq(kw("duration"), $.string),
        seq(kw("decimal"), $.string),
        seq(kw("geometry"), $.object_literal),
        kw("geometry"),
        $.identifier,
        $.record_ref,
        $.window_call,
        $.function_call,
        $.array_literal,
        $.object_literal,
        $.path_expression,
        seq("(", $.expression, ")"),
        prec(1, seq("(", $.select_statement, ")")),
        seq(kw("IF"), $.expression, kw("THEN"), $.expression, kw("ELSE"), $.expression)
      ),

    path_expression: ($) =>
      prec(9, seq(
        choice($.identifier, $.parameter),
        repeat1(choice(seq(".", $.identifier), seq("[", $.expression, "]")))
      )),

    function_call: ($) =>
      prec(10, seq(choice(seq($.identifier, "::", $.identifier), $.identifier), "(", optional(commaSep1(choice($.expression, "*"))), ")")),
window_call: ($) =>
      prec(10, seq(choice(seq($.identifier, "::", $.identifier), $.identifier), "(", optional(commaSep1(choice($.expression, "*"))), ")",
        kw("OVER"), "(",
        optional(seq(kw("PARTITION"), kw("BY"), commaSep1($.expression))),
        optional(seq(kw("ORDER"), kw("BY"), commaSep1($.order_item))),
        ")")),

    array_literal: ($) => seq("[", optional(commaSep1($.expression)), "]"),

    object_literal: ($) => seq("{", optional(commaSep1($.object_pair)), "}"),

    object_pair: ($) =>
      seq(choice($.identifier, $.string, kw("TYPE"), kw("ORDER"), kw("VALUE"), kw("GROUP"),
                  kw("LIMIT"), kw("START"), kw("END"), kw("FOR"), kw("ROLE"), kw("AS")),
        ":", $.expression),
  },
});

function commaSep1(rule) {
  return seq(rule, repeat(seq(",", rule)));
}
