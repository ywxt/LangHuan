mod error;
mod http;
mod package;
mod runtime;
mod schema;

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
