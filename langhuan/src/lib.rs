mod error;
mod package;

pub mod http;
pub mod runtime;
pub mod schema;

pub use error::*;

#[cfg(test)]
pub(crate) mod tests {
    #[macro_export(local_inner_macros)]
    /// a macro to create a hashset
    macro_rules! hashset {
            ( $( $x:expr ),* ) => {
                {
                    let mut set = ::std::collections::HashSet::new();
                    $(
                        set.insert($x);
                    )*
                    set
                }
            };
        }
}
