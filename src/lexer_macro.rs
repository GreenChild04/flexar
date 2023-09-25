/// Creates a lexer for tokenising a file
/// # Example
/// ```rust
/// flexar::compiler_error! {
///    [[Define]]
///    (E001) "invalid character": ((1) "`", "` is an invalid character");
///    (E002) "string not closed": "expected `\"` to close string";
/// }
///
/// #[derive(Debug, PartialEq)]
/// pub enum Lexer {
///    Slash,
///    Plus,
///    LParen,
///    RParen,
///    EE,
///    EEE,
///    EQ,
///    Dot,
///    Colon,
///    Str(String),
///    Int(u32),
///    Float(f32),
/// }
///
/// flexar::lexer! {
///    [[Lexer] flext: Flext, current, 'cycle]
///    else flexar::compiler_error!((E001, flext.cursor.position()) current).throw();
///
///    Slash: /;
///    Plus: +;
///    LParen: '(';
///    RParen: ')';
///    Dot: .;
///    Colon: :;
///    [" \n\t"] >> ({ flext.advance(); flext = flext.spawn(); continue 'cycle; });
///
///    // `=` stuff
///    EEE: (= = =);
///    EE: (= =);
///    EQ: =;
///    '"' child {
///        { child.advance() };
///        set string { String::new() };
///        rsome current {
///            ck (current, '"') {
///               { child.advance() };
///               done Str(string);
///           };
///           { string.push(current) };
///        };
///        throw E002(child.cursor.spawn().position());
///    };
///    ["0123456789"] child {
///        set number { String::new() };
///        set dot false;
///        rsome (current, 'number) {
///            set matched false;
///             ck (current, ["0123456789"]) {
///                 mut matched true;
///                 { number.push(current) };
///             };
///             ck (current, '.') {
///                 if (dot) {
///                     done Float(number.parse().unwrap());
///                 };
///                 mut matched true;
///                 mut dot true;
///                 { number.push(current) };
///             };
///             {if !matched {break 'number}};
///         };
///         if (dot) { done Float(number.parse().unwrap()); };
///         done Int(number.parse().unwrap());
///     };
/// }
#[macro_export]
macro_rules! lexer {
    ([[$name:ty] $flext:ident: $flext_type:ident, $current:ident $(, $label:tt)?] else $no_match:expr; $($first:tt$sep:tt$second:tt;)*) => {
        /// Lexer context for tokenising
        pub struct $flext_type {
            pub cursor: $crate::cursor::MutCursor,
            pub current: Option<char>,
        }
        
        impl $flext_type {
            pub fn new(file_name: String, contents: &str) -> Self {
                let cursor = $crate::cursor::MutCursor::new($crate::cursor::Cursor::new(file_name, contents));
                let $current = cursor.pos_end.get_char();
                Self {
                    cursor,
                    $current,
                }
            }

            /// Advances to the next token
            pub fn advance(&mut self) {
                self.cursor.advance();
                self.current = self.cursor.current_char;
            }

            /// Spawns a child flext
            pub fn spawn(&self) -> Self {
                Self {
                    cursor: self.cursor.spawn(),
                    current: self.current,
                }
            }
        }

        impl $name {
            pub fn tokenise(mut $flext: $flext_type) -> Box<[$name]> {
                let mut tokens = Vec::<Self>::new();
                $($label:)? while let Some($current) = $flext.current {
                    tokens.push('code: {
                        $($crate::lexer!(@sect $flext 'code $current $first$sep$second);)*
                        $no_match
                    });
                    $flext.cursor.pos_start = $flext.cursor.pos_end.clone(); // cause different tokens with different start pos
                } tokens.into_boxed_slice()
            }
        }
    };

    // Sections

    (@sect $flext:ident $label:tt $current:ident $out:ident: ($char:tt $($tail:tt)*)) => { // Change to something more efficient if too slow
        if $crate::lexer!(@value $current $char) {
            let mut child = $flext.spawn();
            child.advance();
            $crate::lexer!(@recur-sect1 $label $out $flext child $($tail)*);
        }
    };

    (@sect $flext:ident $label:tt $current:ident $name:ident: $char:tt) => {
        if $crate::lexer!(@value $current $char) {
            $flext.advance();
            break $label Self::$name;
        }
    };

    (@sect $flext:ident $label:tt $current:ident $start:tt $child:ident {$($($code:block)? $($key:ident $param:tt $body:tt)?;)*}) => {
        if $crate::lexer!(@value $current $start) {
            let mut $child = $flext.spawn();
            $(
                $($crate::lexer!(@det $child $flext $label $key $param $body);)?
                $($code;)?
            )*
        }
    };

    (@sect $flext:ident $label:tt $current:ident $char:tt >> ($action:expr)) => {
        if $crate::lexer!(@value $current $char) {
            $action;
        }
    };

    // Recur

        // For section 1
        (@recur-sect1 $label:tt $out:ident $flext:ident $child:ident $char:tt) => {
            if let Some(current) = $child.current {
                if $crate::lexer!(@value current $char) {
                    $flext = $child;
                    $flext.advance();
                    break $label Self::$out;
                }
            }
        };

        (@recur-sect1 $label:tt $out:ident $flext:ident $child:ident $char:tt $($tail:tt)*) => {
            if let Some(current) = $child.current {
                if $crate::lexer!(@value current $char) {
                    let mut child = $child.spawn();
                    child.advance();
                    $crate::lexer!(@recur-sect1 $label $out $flext child $($tail)*);
                }
            }
        };

    // Detailed

    (@det $child:ident $flext:ident $label:tt ck ($current:ident, $val:tt) {$($($code:block)? $($key:ident $param:tt $body:tt)?;)*}) => {
        if $crate::lexer!(@value $current $val) {
            $(
                $($crate::lexer!(@det $child $flext $label $key $param $body);)?
                $($code;)?
            )*
        }
    };

    (@det $child:ident $flext:ident $label:tt if ($condition:expr) {$($($code:block)? $($key:ident $param:tt $body:tt)?;)*}) => {
        if $condition {
            $(
                $($crate::lexer!(@det $child $flext $label $key $param $body);)?
                $($code;)?
            )*
        }
    };

    (@det $child:ident $flext:ident $label:tt scope $name:ident {$($($code:block)? $($key:ident $param:tt $body:tt)?;)*}) => {
        {
            let mut $name = $child.spawn();
            $name.advance();
            $(
                $($crate::lexer!(@det $child $flext $label $key $param $body);)?
                $($code;)?
            )*
        }
    };

    (@det $child:ident $flext:ident $label:tt done $var:ident ($($spec:expr)?)) => {
        $flext = $child;
        break $label Self::$var$(($spec))?;
    };

    (@det $child:ident $flext:ident $label:tt set $var:ident $val:expr) => {
        let mut $var = $val;
    };

    (@det $child:ident $flext:ident $label:tt mut $var:ident $val:expr) => {
        $var = $val;
    };

    (@det $child:ident $flext:ident $label:tt throw $err:ident ($position:expr $(,$spec:tt)?)) => {
        let _: () = $crate::compiler_error!(($err, $position) $($spec)?).throw();
    };

    (@det $child:ident $flext:ident $label:tt rsome $current:ident {$($($code:block)? $($key:ident $param:tt $body:tt)?;)*}) => {
        while let Some($current) = $child.current {
            $(
                $($crate::lexer!(@det $child $flext $label $key $param $body);)?
                $($code;)?
            )*
            $child.advance();
        }
    };

    (@det $child:ident $flext:ident $label:tt rsome ($current:ident, $while_label:tt) {$($($code:block)? $($key:ident $param:tt $body:tt)?;)*}) => {
        $while_label: while let Some($current) = $child.current {
            $(
                $($crate::lexer!(@det $child $flext $label $key $param $body);)?
                $($code;)?
            )*
            $child.advance();
        }
    };

    (@det $child:ident $flext:ident $label:tt some $current:ident {$($($code:block)? $($key:ident $param:tt $body:tt)?;)*}) => {
        if let Some($current) = $child.current {
            $(
                $($crate::lexer!(@det $child $flext $label $key $param $body);)?
                $($code;)?
            )*
            $child.advance();
        }
    };

    (@det $child:ident $flext:ident $label:tt $invalid:ident $val:tt $var:tt) => {
        compile_error!(concat!("[lexer] invalid detailed instruction `", stringify!($invalid), "`"))
    };

    // Values

    (@value $current:ident [$val:literal]) => {
        $val.contains($current)
    };

    (@value $current:ident $val:literal) => {
        $current == $val
    };

    (@value $current:ident $val:tt) => {
        stringify!($val).contains($current)
    };
}