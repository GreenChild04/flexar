/// Creates a parsing function for a node
#[macro_export]
macro_rules! parser {
    ([[$node:ty] $parxt:ident: $token:ident] $($func:ident {$($($pats:tt),* => $body:tt$end:tt)*} else $else:ident$else_body:tt;)*) => {
        #[allow(clippy::diverging_sub_expression)]
        impl $node {
            $(pub fn $func($parxt: &mut $crate::parxt::Parxt<'_, $token>) -> Result<$crate::token_node::Node<$node>, (u8, $crate::compile_error::CompileError)> {
                let mut last_error: Option<(u8, $crate::compile_error::CompileError)> = None;
                let start_pos = $parxt.position();
                let mut child = $parxt.spawn();

                $($crate::parser!(@req $token start_pos $parxt child last_error 0, $($pats),* => $body$end);)*
                
                if let Some((i, x)) = last_error { if i > 0 { return Err((i, x)); } }
                Err($crate::parser!(@else start_pos $parxt $else$else_body 0))
            })*
        }
    };

    // Requirements
    (@req $token:ident $start_pos:ident $parxt:ident $child:ident $last_error:ident $depth:expr, [$out:ident: $func:expr], $($tail:tt),* => $body:tt$end:tt) => {
        #[allow(unused_parens)]
        match $func(&mut $child) {
            Ok($out) => {
                $crate::parser!(@req $token $start_pos $parxt $child $last_error $depth + 1, $($tail),* => $body$end);
            }
            Err((i, x)) => {
                let i = i + $depth;
                match $last_error {
                    None => $last_error = Some((i, x)),
                    Some((ii, _)) => if i > ii {
                        $last_error = Some((i, x));
                    },
                }
            },
        };
    };

    (@req $token:ident $start_pos:ident $parxt:ident $child:ident $last_error:ident $depth:expr, [$out:ident: $func:expr] => $body:tt$end:tt) => {
        #[allow(unused_parens)]
        match $func(&mut $child) {
            Ok($out) => {
                $crate::parser!(@body $token $start_pos $parxt $child $last_error $body$end | $depth + 1);
            }
            Err((i, x)) => {
                let i = i + $depth;
                match $last_error {
                    None => $last_error = Some((i, x)),
                    Some((ii, _)) => if i > ii {
                        $last_error = Some((i, x));
                    },
                }
            },
        };
    };

    (@req $token:ident $start_pos:ident $parxt:ident $child:ident $last_error:ident $depth:expr, ($head:ident$($neck:tt)?), $($tail:tt),* => $body:tt$end:tt) => {
        #[allow(unused_parens)]
        if let Some($token::$head$($neck)?) = $child.current() {
            $child.advance();
            $crate::parser!(@req $token $start_pos $parxt $child $last_error $depth + 1, $($tail),* => $body$end);
        }
    };

    (@req $token:ident $start_pos:ident $parxt:ident $child:ident $last_error:ident $depth:expr, ($head:ident$($neck:tt)?) => $body:tt$end:tt) => {
        #[allow(unused_parens)]
        if let Some($token::$head$($neck)?) = $child.current() {
            $child.advance();
            $crate::parser!(@body $token $start_pos $parxt $child $last_error $body$end | $depth + 1);
        }
    };

    // Body
    (@body $token:ident $start_pos:ident $parxt:ident $child:ident $last_error:ident {$($($pats:tt),* => $body:tt$end:tt)*} $((else $else:ident$else_body:tt))? $(;)? | $depth:expr) => {
        let mut last_error: Option<(u8, $crate::compile_error::CompileError)> = None;
        let mut child = $child.spawn();
        
        $($crate::parser!(@req $token $start_pos $parxt child $last_error $depth, $($pats),* => $body$end);)*
        if let Some((i, x)) = last_error {
            $last_error = Some((i, x));
        }
        $(#[allow(unreachable_code)]
        match $last_error {
            Some((i, _)) if i > $depth => *$parxt = $child.clone(), // if things break remove this,
            _ => {
                *$parxt = $child.clone(); // if things break remove this
                $last_error = Some($crate::parser!(@else $start_pos $parxt $else$else_body $depth));
            },
        })?
    };

    (@body $token:ident $start_pos:ident $parxt:ident $child:ident $last_error:ident ($node:ident$($data:tt)?); | $depth:expr) => {
        let pos = $parxt.position();
        *$parxt = $child.clone();
        return Ok($crate::token_node::Node::new($start_pos.combine(&pos), Self::$node$($data)?));
    };

    (@body $token:ident $start_pos:ident $parxt:ident $child:ident $last_error:ident ($data:tt); | $depth:expr) => {
        let pos = $parxt.position();
        *$parxt = $child.clone();
        return Ok($crate::token_node::Node::new($start_pos.combine(&pos), Self$data));
    };

    (@body $token:ident $start_pos:ident $parxt:ident $child:ident $last_error:ident [$node:expr]; | $depth:expr) => {
        *$parxt = $child.clone();
        return Ok($node);
    };

    // Else
    (@else $start_pos:ident $parxt:ident Err($err:ident $(:$($rest:expr),+)?) $depth:expr) => {
        ($depth, $crate::compiler_error!(($err, $parxt.position()) $($($rest),*)?))
    };

    (@else $start_pos:ident $parxt:ident Ok($else:expr) $depth:expr) => {
        return Ok($crate::token_node::Node::new($start_pos, $else));
    };

    (@else $start_pos:ident $parxt:ident Raw($raw:expr) $depth:expr) => {
        return $raw;
    };

    (@else $start_pos:ident $parxt:ident Other($variant:ident $else:expr) $depth:expr) => {
        match $else {
            Ok(x) => return Ok($crate::token_node::Node::new($start_pos.combine(&x.position), Self::$variant(x))),
            Err((i, x)) => ((i + $depth, x)),
        }
    };

    // Outputs
    (@output $variant:ident $rest:tt) => {
        Self::$variant$crate::parser!(@output $rest)
    };

    (@output $rest:tt) => {
        Self$rest
    };
}