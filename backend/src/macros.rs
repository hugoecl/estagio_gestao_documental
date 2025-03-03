#[macro_export]
macro_rules! impl_enum_conversions {
    ($enum_name:ident, $first_variant:ident => $first_value:expr, $($variant:ident => $value:expr),* $(,)?) => {
        impl From<i8> for $enum_name {
            fn from(value: i8) -> Self {
                match value {
                    $first_value => $enum_name::$first_variant,
                    $($value => $enum_name::$variant,)*
                    _ => $enum_name::$first_variant // Default to first variant
                }
            }
        }

        impl std::str::FromStr for $enum_name {
            type Err = ();

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    stringify!($first_value) => Ok($enum_name::$first_variant),
                    $(stringify!($value) => Ok($enum_name::$variant),)*
                    _ => Err(())
                }
            }
        }
    };
}

// #[macro_export]
// macro_rules! impl_enum_conversions {
//     ($enum_name:ident { $first_variant:ident, $($variant:ident),* $(,)? }) => {
//         impl From<i8> for $enum_name {
//             fn from(value: i8) -> Self {
//                 match value {
//                     0 => $enum_name::$first_variant,
//                     $(
//                         count!($first_variant $($variant)*) => $enum_name::$variant,
//                     )*
//                     _ => $enum_name::$first_variant // Default to first variant
//                 }
//             }
//         }

//         impl std::str::FromStr for $enum_name {
//             type Err = ();

//             fn from_str(s: &str) -> Result<Self, Self::Err> {
//                 match s {
//                     "0" => Ok($enum_name::$first_variant),
//                     $(
//                         stringify!(count!($first_variant $($variant)*)) => Ok($enum_name::$variant),
//                     )*
//                     _ => Err(())
//                 }
//             }
//         }
//     };
// }

// // Helper macro to count indices
// macro_rules! count {
//     () => { 0 };
//     ($head:ident $($tail:ident)*) => { 1 + count!($($tail)*) };
// }
