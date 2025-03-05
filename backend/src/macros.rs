// TODO: Make the macros better by not having to pass in the variants

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


        impl serde::ser::Serialize for $enum_name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                match self {
                    $enum_name::$first_variant => serializer.serialize_i8($first_value),
                    $($enum_name::$variant => serializer.serialize_i8($value),)*
                }
            }
        }

    };
}
