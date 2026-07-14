pub const PROMPT: &'static str = "\x1b[35m::\x1b[0m ";
pub const ACTIVATE_FACETS: &'static str = "activate";
pub const ACTIVATE_FACETS_LT: &'static str = "activate!";
pub const ACTIVATE_FACETS_LAZY: &'static str = "lazy+";
pub const SHOW_FACETS: &'static str = "facets";
pub const COMPUTE_FACETS_SU: &'static str = "compute^";
pub const COMPUTE_FACETS: &'static str = "compute";
pub const FACET_COUNT: &'static str = "count";
pub const FACET_COUNTS: &'static str = "counts";
pub const FACET_COUNTS_PROJECTING: &'static str = "counts!";
pub const WEIGHTED_FACET_COUNT: &'static str = "wcount";
pub const WEIGHTED_FACET_COUNTS: &'static str = "wcounts";
pub const ANSWER_SET_COUNT: &'static str = "solvecount";
pub const ANSWER_SET_COUNTS: &'static str = "solvecounts";
pub const ENUMERATE_SOLUTIONS: &'static str = "solve";
pub const ENUMERATE_PROJECTED_SOLUTIONS: &'static str = "solve*";
pub const SHOW_ROUTE: &'static str = "route";
pub const CLEAR_ROUTE: &'static str = "clear";
pub const DEL_LAST: &'static str = "undo";
pub const CHANGE_MODE: &'static str = ":m";
pub const DISPLAY_MODE: &'static str = ":mode";
pub const PROPOSE_STEP: &'static str = "propose";
pub const TAKE_STEP: &'static str = "step";
pub const SHOW_PROGRAM: &'static str = ":src";
pub const SHOW_ATOMS: &'static str = ":atoms";
pub const FILTER_ATOMS: &'static str = ":filter_atoms";
pub const IS_ATOM: &'static str = ":isatom";
pub const SOE: &'static str = ":soe";
pub const CONTEXT: &'static str = ":ctx";
pub const CACHE: &'static str = "cache";
pub const OPTIMIZATION: &'static str = "optimization";
pub const SIGNIFICANCE: &'static str = "sig";
pub const SIGNIFICANCE_PROJECTING: &'static str = "sig*";
pub const COMPUTE_FACETS_SOE: &'static str = "!?soe";
pub const MANUAL: &'static str = "man";
pub const IS_FACET: &'static str = "isfacet";
pub const IS_FACET_R: &'static str = "isfacet!";
pub const ENTAILMENT: &'static str = "|=";
pub const QUIT: &'static str = ":q";
pub const LOOP: &'static str = "while";
pub const WHILE_LOOP_DO: &'static str = "do";
pub const WHILE_LOOP_CMD_SEP: &'static str = ";";
pub const WHILE_LOOP_VAR_FACETS: &'static str = "#facets";
pub const WHILE_LOOP_VAR_ROUTE: &'static str = "#route";
pub const WHILE_LOOP_OP_NEQ: &'static str = "!=";
pub const WHILE_LOOP_OP_GT: &'static str = ">";
pub const WHILE_LOOP_OP_GTE: &'static str = ">=";
pub const WHILE_LOOP_OP_LT: &'static str = "<";
pub const WHILE_LOOP_OP_LTE: &'static str = "<=";

// Hard-Coded Aliase für Rückwartskompatibilität
pub const ACTIVATE_FACETS_ALIAS: &'static str = "+";
pub const ACTIVATE_FACETS_LT_ALIAS: &'static str = "+'";
pub const ACTIVATE_FACETS_LAZY_ALIAS: &'static str = ":+";
pub const SHOW_FACETS_ALIAS: &'static str = "?";
pub const COMPUTE_FACETS_SU_ALIAS: &'static str = "'!?";
pub const COMPUTE_FACETS_ALIAS: &'static str = "!?";
pub const FACET_COUNT_ALIAS: &'static str = "#?";
pub const FACET_COUNTS_ALIAS: &'static str = "#??";
pub const FACET_COUNTS_PROJECTING_ALIAS: &'static str = "!#??";
pub const WEIGHTED_FACET_COUNT_ALIAS: &'static str = "#?w";
pub const WEIGHTED_FACET_COUNTS_ALIAS: &'static str = "#??w";
pub const ANSWER_SET_COUNT_ALIAS: &'static str = "#!";
pub const ANSWER_SET_COUNTS_ALIAS: &'static str = "#!!";
pub const ENUMERATE_SOLUTIONS_ALIAS: &'static str = "!";
pub const ENUMERATE_PROJECTED_SOLUTIONS_ALIAS: &'static str = "!*";
pub const SHOW_ROUTE_ALIAS: &'static str = "@";
pub const CLEAR_ROUTE_ALIAS: &'static str = "--";
pub const DEL_LAST_ALIAS: &'static str = "-";
pub const CHANGE_MODE_ALIAS: &'static str = "'";
pub const PROPOSE_STEP_ALIAS: &'static str = "$";
pub const TAKE_STEP_ALIAS: &'static str = "$$";
pub const CONTEXT_ALIAS: &'static str = ">";
pub const CACHE_ALIAS: &'static str = ":cache";
pub const OPTIMIZATION_ALIAS: &'static str = ":opt";
pub const SIGNIFICANCE_ALIAS: &'static str = "%";
pub const SIGNIFICANCE_PROJECTING_ALIAS: &'static str = "!%";
pub const IS_FACET_ALIAS: &'static str = ":?";
pub const IS_FACET_R_ALIAS: &'static str = ":?r";

pub const FILTER_KEYWORD: &'static str = "%filter ";
pub const CONTROL_ON: &'static str = "on";
pub const CONTROL_OFF: &'static str = "off";
pub const CONTROL_CLEAR: &'static str = "clear";
pub const CONTROL_SIZE: &'static str = "size";
pub const CONTROL_STATUS: &'static str = "status";

pub(crate) fn manual() {
    println!(
        "display facet-inducing atoms                                                                  ->  {SHOW_FACETS}"
    );
    println!(
        "display route                                                                                 ->  {SHOW_ROUTE}"
    );
    println!(
        "enumerate n=[int] answer sets                                                                 ->  {ENUMERATE_SOLUTIONS} n"
    );
    println!(
        "activate facets=[whitespace seperated literals, e.g., a ~b]                                   ->  {ACTIVATE_FACETS} facets"
    );
    println!(
        "activate facets=[whitespace seperated literals, e.g., a ~b] and display consequences          ->  {ACTIVATE_FACETS_LT} facets"
    );
    println!(
        "deactivate previous facet                                                                     ->  {DEL_LAST}"
    );
    println!(
        "deactivate all facets                                                                         ->  {CLEAR_ROUTE}"
    );
    //println!("check whether regex-matched atoms are facets                                                  ->  {IS_FACET} regex");
    //println!("check whether regex-matched atoms are facets with one consistency check                       ->  {IS_FACET_R} regex");
    println!(
        "declare cnf=[e.g., a|~b&c|d] context/query                                                    ->  {CONTEXT} cnf"
    );
    println!(
        "select navigation mode=[{{{{min,max}}#{{f,a,s}}, go}}]                                              ->  {CHANGE_MODE} mode"
    );
    println!(
        "next step in mode                                                                             ->  {PROPOSE_STEP}"
    );
    println!(
        "perform next step in mode                                                                     ->  {TAKE_STEP}"
    );
    println!(
        "compute facets among atoms that match targets=[regex]                                         ->  {COMPUTE_FACETS} targets"
    );
    println!(
        "facet count of facet                                                                          ->  {FACET_COUNT} facet"
    );
    println!(
        "facet counts under each facet filtered from current facets by regex                           ->  {FACET_COUNTS} regex"
    );
    //println!("facet count w.r.t. to provided weights in given filename                                      ->  {WEIGHTED_FACET_COUNT} filename facet");
    //println!("facet counts under each facet w.r.t. to provided regex and weights in given filename          ->  {WEIGHTED_FACET_COUNTS} filename regex");
    println!(
        "facet counts under each facet in targets_1=[regex] using projection on targets_2=[regex]      ->  {FACET_COUNTS_PROJECTING} targets_2 targets_1"
    );
    println!(
        "significance of facets=[regex] for some literal=[a or ~a]                                     ->  {SIGNIFICANCE} literal facets"
    );
    println!(
        "significance of facets=[regex] for some literal=[a or ~a] using projection on targets=[regex] ->  {SIGNIFICANCE_PROJECTING} literal targets facets "
    );
    println!(
        "answer set count                                                                              ->  {ANSWER_SET_COUNT}"
    );
    println!(
        "answer set counts under each facet                                                            ->  {ANSWER_SET_COUNTS}"
    );
    println!(
        "enumerate representative answer sets regarding targets=[regex] filtered from current facets   ->  {SOE} targets"
    );
    println!(
        "display program                                                                               ->  {SHOW_PROGRAM}"
    );
    println!(
        "display atoms                                                                                 ->  {SHOW_ATOMS}"
    );
    println!(
        "display regex-matched atoms                                                                   ->  {FILTER_ATOMS} regex"
    );
    println!(
        "atom check                                                                                    ->  {IS_ATOM}"
    );
    println!(
        "display navigation mode                                                                       ->  {DISPLAY_MODE}"
    );
    println!(
        "control facet cache                                                                            ->  {CACHE} {{{CONTROL_ON},{CONTROL_OFF},{CONTROL_CLEAR},{CONTROL_SIZE},{CONTROL_STATUS}}}"
    );
    println!(
        "control optimized paths                                                                        ->  {OPTIMIZATION} {{{CONTROL_ON},{CONTROL_OFF},{CONTROL_STATUS}}}"
    );
    println!(
        "quit                                                                                          ->  {QUIT}"
    );
    //println!("see documentation for more details");
}
