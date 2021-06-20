/// Parses matcher syntax on behalf of rule!()
///
/// Note: All branches return a Vec to enable uniform treatment inside rule! and
/// because the Literal branch can expand to multiple Literals
#[macro_export]
#[doc(hidden)]
macro_rules! symbol {
    // Rule syntax is a bareword (ident doesn't cover all legal possibilities
    // but does cover all sensible ones). The output matcher contains the
    // stringified rule name.
    ($rule:ident) => {
        vec![$crate::grammar::Symbol::Rule(String::from(
            ::std::stringify!($rule),
        ))]
    };
    // OneOf is a string literal surrounded by [] (which is conveniently similar
    // to a regex character class and makes the whole thing one token tree in
    // rule!)
    ([ $str:literal ]) => {
        vec![$crate::grammar::Symbol::OneOf(
            $str.chars().collect::<::std::collections::HashSet<_>>(),
        )]
    };
    // A string literal without [] is a sequence of Literal matchers (one for
    // each character in the string)
    ($str:literal) => {
        $str.chars()
            .map($crate::grammar::Symbol::Literal)
            .collect::<Vec<_>>()
    };
}

/// Parses a rule body
#[macro_export]
#[doc(hidden)]
macro_rules! symbols {
    // Required because the general case that flattens a Vec<Vec<Symbol>> fails
    // to type check if the outer Vec is empty
    () => { Vec::new() };
    ($($symbols:tt),+) => {
        vec![
            // symbol! is used to parse each token tree in the body, the
            // result is a Vec<Vec<Matcher>> which has to be flattened for
            // Rule::new
            $($crate::symbol!($symbols)),*
        ].into_iter().flatten().collect::<Vec<_>>()
    }
}

/// Parses a single rule (without the trailing ;) on behalf of grammar! { }
#[macro_export]
#[doc(hidden)]
macro_rules! rule {
    // Rule syntax is RuleName -> body
    // The rule name is a bareword as in matcher!() above.
    // Conveniently (from symbol!()) all of the possible symbol syntaxes are
    // parsed as a single token tree so the rule body can be a (possibly empty)
    // list of token trees.
    ($name:ident -> $($symbols:tt)*) => {
        $crate::grammar::Rule::new(
            String::from(::std::stringify!($name)),
            // Auxiliary macro to handle empty rules properly
            $crate::symbols!($($symbols),*)
        )
    }
}

/// Helper for grammar! { }. Collects rules by finding each ; then passing the
/// preceding token trees to rule!().
#[macro_export]
#[doc(hidden)]
macro_rules! grammar_aux {
    // Base case: Found all of the rules and don't have any leftover tokens,
    // construct a new grammar.
    ([][$($rules:expr)*]) => {
        $crate::grammar::Grammar::new(vec![$($rules),*])
    };
    // No more tokens in the input but there are still some in the
    // accumulator. Assume that they represent a rule (this is caused by missing
    // the ; from the last rule).
    ([$($rule:tt)+][$($rules:expr)*]) => {
        $crate::grammar_aux!([][$($rules)* $crate::rule!($($rule)*)]);
    };
    // Found a ;. Assume everything preceding it (now in the accumulator) is one
    // rule. The rule is constructed with rule! then pushed onto the rules list
    ([$($rule:tt)*][$($rules:expr)*] ; $($rest:tt)*) => {
        $crate::grammar_aux!([][$($rules)* $crate::rule!($($rule)*)] $($rest)*)
    };
    // Something other than a ;. Push it onto the accumulator then recuse on the
    // remaining input.
    ([$($acc:tt)*][$($rules:expr)*] $first:tt $($rest:tt)*) => {
        $crate::grammar_aux!([$($acc)* $first][$($rules)*] $($rest)*)
    };
}

/// Macro to construct a [Grammar]. Expands to appropriate calls to
/// [Grammar::new] and [Rule::new]
///
/// # Syntax
/// ```ignore
/// grammar! {
///     <Rule Name> -> <Rule Body>;
///     ...
/// }
/// ```
///
/// # Panics
/// See [Grammar::new] and [Rule::new]
///
/// # Examples
/// Within the rule body an unquoted rule name becomes [Symbol::Rule]
/// ```
/// # use parsey::grammar;
/// # use parsey::grammar::{ Grammar, Rule, Symbol };
/// assert_eq!(
///     grammar! {
///         Rule -> AnotherRule;
///     },
///     Grammar::new(vec![
///         Rule::new(
///             String::from("Rule"),
///             vec![Symbol::Rule(String::from("AnotherRule"))]
///         )
///     ])
/// )
/// ```
/// A bare string becomes a sequence of [Symbol::Literal] (one for each character)
/// ```
/// # use parsey::grammar;
/// # use parsey::grammar::{ Grammar, Rule, Symbol };
/// assert_eq!(
///     grammar! {
///         Rule -> "12";
///     },
///     Grammar::new(vec![
///         Rule::new(
///             String::from("Rule"),
///             vec![Symbol::Literal('1'), Symbol::Literal('2')]
///         )
///     ])
/// )
/// ```
/// A string wrapped in `[]` is [Symbol::OneOf]
/// ```
/// # use std::collections::HashSet;
/// # use parsey::grammar;
/// # use parsey::grammar::{ Grammar, Rule, Symbol };
/// assert_eq!(
///     grammar! {
///         Rule -> ["12"];
///     },
///     Grammar::new(vec![
///         Rule::new(
///             String::from("Rule"),
///             vec![Symbol::OneOf(vec!['1','2'].into_iter().collect::<HashSet<_>>())]
///         )
///     ])
/// )
/// ```
///
/// [Grammar]: super::Grammar
/// [Grammar::new]: super::Grammar::new
/// [Rule::new]: super::Rule::new
/// [Symbol::OneOf]: super::Symbol::OneOf
/// [Symbol::Literal]: super::Symbol::Literal
/// [Symbol::Rule]: super::Symbol::Rule
#[macro_export]
macro_rules! grammar {
    ($($rules:tt)+) => {
        // Construct the new grammar with grammar_aux initialised with two empty
        // accumulators
        $crate::grammar_aux!([][] $($rules)+)
    };
}

#[cfg(test)]
macro_rules! hashset {
    ($($e:expr),*) => {
        vec![$($e),*].into_iter().collect::<::std::collections::HashSet<_>>()
    }
}
