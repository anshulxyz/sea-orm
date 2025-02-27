use crate::{ColumnDef, DbErr, Iterable, TryGetable};
use sea_query::{Nullable, Value, ValueType};

/// A Rust representation of enum defined in database.
///
/// # Implementations
///
/// You can implement [ActiveEnum] manually by hand or use the derive macro [DeriveActiveEnum](sea_orm_macros::DeriveActiveEnum).
///
/// # Examples
///
/// Implementing it manually versus using the derive macro [DeriveActiveEnum](sea_orm_macros::DeriveActiveEnum).
///
/// > See [DeriveActiveEnum](sea_orm_macros::DeriveActiveEnum) for the full specification of macro attributes.
///
/// ```rust
/// use sea_orm::entity::prelude::*;
///
/// // Using the derive macro
/// #[derive(Debug, PartialEq, EnumIter, DeriveActiveEnum)]
/// #[sea_orm(
///     rs_type = "String",
///     db_type = "String(Some(1))",
///     enum_name = "category"
/// )]
/// pub enum DeriveCategory {
///     #[sea_orm(string_value = "B")]
///     Big,
///     #[sea_orm(string_value = "S")]
///     Small,
/// }
///
/// // Implementing it manually
/// #[derive(Debug, PartialEq, EnumIter)]
/// pub enum Category {
///     Big,
///     Small,
/// }
///
/// impl ActiveEnum for Category {
///     // The macro attribute `rs_type` is being pasted here
///     type Value = String;
///
///     // Will be atomically generated by `DeriveActiveEnum`
///     fn name() -> String {
///         "category".to_owned()
///     }
///
///     // Will be atomically generated by `DeriveActiveEnum`
///     fn to_value(&self) -> Self::Value {
///         match self {
///             Self::Big => "B",
///             Self::Small => "S",
///         }
///         .to_owned()
///     }
///
///     // Will be atomically generated by `DeriveActiveEnum`
///     fn try_from_value(v: &Self::Value) -> Result<Self, DbErr> {
///         match v.as_ref() {
///             "B" => Ok(Self::Big),
///             "S" => Ok(Self::Small),
///             _ => Err(DbErr::Type(format!(
///                 "unexpected value for Category enum: {}",
///                 v
///             ))),
///         }
///     }
///
///     fn db_type() -> ColumnDef {
///         // The macro attribute `db_type` is being pasted here
///         ColumnType::String(Some(1)).def()
///     }
/// }
/// ```
///
/// Using [ActiveEnum] on Model.
///
/// ```
/// use sea_orm::entity::prelude::*;
///
/// // Define the `Category` active enum
/// #[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
/// #[sea_orm(rs_type = "String", db_type = "String(Some(1))")]
/// pub enum Category {
///     #[sea_orm(string_value = "B")]
///     Big,
///     #[sea_orm(string_value = "S")]
///     Small,
/// }
///
/// #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
/// #[sea_orm(table_name = "active_enum")]
/// pub struct Model {
///     #[sea_orm(primary_key)]
///     pub id: i32,
///     // Represents a db column using `Category` active enum
///     pub category: Category,
///     pub category_opt: Option<Category>,
/// }
///
/// #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
/// pub enum Relation {}
///
/// impl ActiveModelBehavior for ActiveModel {}
/// ```
pub trait ActiveEnum: Sized + Iterable {
    /// Define the Rust type that each enum variant represents.
    type Value: Into<Value> + ValueType + Nullable + TryGetable;

    /// Get the name of enum
    fn name() -> String;

    /// Convert enum variant into the corresponding value.
    fn to_value(&self) -> Self::Value;

    /// Try to convert the corresponding value into enum variant.
    fn try_from_value(v: &Self::Value) -> Result<Self, DbErr>;

    /// Get the database column definition of this active enum.
    fn db_type() -> ColumnDef;

    /// Convert an owned enum variant into the corresponding value.
    fn into_value(self) -> Self::Value {
        Self::to_value(&self)
    }

    /// Get the name of all enum variants
    fn values() -> Vec<Self::Value> {
        Self::iter().map(Self::into_value).collect()
    }
}

#[cfg(test)]
mod tests {
    use crate as sea_orm;
    use crate::{entity::prelude::*, *};
    use pretty_assertions::assert_eq;

    #[test]
    fn active_enum_string() {
        #[derive(Debug, PartialEq, Eq, EnumIter)]
        pub enum Category {
            Big,
            Small,
        }

        impl ActiveEnum for Category {
            type Value = String;

            fn name() -> String {
                "category".to_owned()
            }

            fn to_value(&self) -> Self::Value {
                match self {
                    Self::Big => "B",
                    Self::Small => "S",
                }
                .to_owned()
            }

            fn try_from_value(v: &Self::Value) -> Result<Self, DbErr> {
                match v.as_ref() {
                    "B" => Ok(Self::Big),
                    "S" => Ok(Self::Small),
                    _ => Err(DbErr::Type(format!(
                        "unexpected value for Category enum: {}",
                        v
                    ))),
                }
            }

            fn db_type() -> ColumnDef {
                ColumnType::String(Some(1)).def()
            }
        }

        #[derive(Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
        #[sea_orm(
            rs_type = "String",
            db_type = "String(Some(1))",
            enum_name = "category"
        )]
        pub enum DeriveCategory {
            #[sea_orm(string_value = "B")]
            Big,
            #[sea_orm(string_value = "S")]
            Small,
        }

        assert_eq!(Category::Big.to_value(), "B".to_owned());
        assert_eq!(Category::Small.to_value(), "S".to_owned());
        assert_eq!(DeriveCategory::Big.to_value(), "B".to_owned());
        assert_eq!(DeriveCategory::Small.to_value(), "S".to_owned());

        assert_eq!(
            Category::try_from_value(&"A".to_owned()).err(),
            Some(DbErr::Type(
                "unexpected value for Category enum: A".to_owned()
            ))
        );
        assert_eq!(
            Category::try_from_value(&"B".to_owned()).ok(),
            Some(Category::Big)
        );
        assert_eq!(
            Category::try_from_value(&"S".to_owned()).ok(),
            Some(Category::Small)
        );
        assert_eq!(
            DeriveCategory::try_from_value(&"A".to_owned()).err(),
            Some(DbErr::Type(
                "unexpected value for DeriveCategory enum: A".to_owned()
            ))
        );
        assert_eq!(
            DeriveCategory::try_from_value(&"B".to_owned()).ok(),
            Some(DeriveCategory::Big)
        );
        assert_eq!(
            DeriveCategory::try_from_value(&"S".to_owned()).ok(),
            Some(DeriveCategory::Small)
        );

        assert_eq!(Category::db_type(), ColumnType::String(Some(1)).def());
        assert_eq!(DeriveCategory::db_type(), ColumnType::String(Some(1)).def());

        assert_eq!(Category::name(), DeriveCategory::name());
        assert_eq!(Category::values(), DeriveCategory::values());
    }

    #[test]
    fn active_enum_derive_signed_integers() {
        macro_rules! test_num_value_int {
            ($ident: ident, $rs_type: expr, $db_type: expr, $col_def: ident) => {
                #[derive(Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
                #[sea_orm(rs_type = $rs_type, db_type = $db_type)]
                pub enum $ident {
                    #[sea_orm(num_value = -10)]
                    Negative,
                    #[sea_orm(num_value = 1)]
                    Big,
                    #[sea_orm(num_value = 0)]
                    Small,
                }

                test_int!($ident, $rs_type, $db_type, $col_def);
            };
        }

        macro_rules! test_fallback_int {
            ($ident: ident, $fallback_type: ident, $rs_type: expr, $db_type: expr, $col_def: ident) => {
                #[derive(Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
                #[sea_orm(rs_type = $rs_type, db_type = $db_type)]
                #[repr(i32)]
                pub enum $ident {
                    Big = 1,
                    Small = 0,
                    Negative = -10,
                }

                test_int!($ident, $rs_type, $db_type, $col_def);
            };
        }

        macro_rules! test_int {
            ($ident: ident, $rs_type: expr, $db_type: expr, $col_def: ident) => {
                assert_eq!($ident::Big.to_value(), 1);
                assert_eq!($ident::Small.to_value(), 0);
                assert_eq!($ident::Negative.to_value(), -10);

                assert_eq!($ident::try_from_value(&1).ok(), Some($ident::Big));
                assert_eq!($ident::try_from_value(&0).ok(), Some($ident::Small));
                assert_eq!($ident::try_from_value(&-10).ok(), Some($ident::Negative));
                assert_eq!(
                    $ident::try_from_value(&2).err(),
                    Some(DbErr::Type(format!(
                        "unexpected value for {} enum: 2",
                        stringify!($ident)
                    )))
                );

                assert_eq!($ident::db_type(), ColumnType::$col_def.def());
            };
        }

        test_num_value_int!(I8, "i8", "TinyInteger", TinyInteger);
        test_num_value_int!(I16, "i16", "SmallInteger", SmallInteger);
        test_num_value_int!(I32, "i32", "Integer", Integer);
        test_num_value_int!(I64, "i64", "BigInteger", BigInteger);

        test_fallback_int!(I8Fallback, i8, "i8", "TinyInteger", TinyInteger);
        test_fallback_int!(I16Fallback, i16, "i16", "SmallInteger", SmallInteger);
        test_fallback_int!(I32Fallback, i32, "i32", "Integer", Integer);
        test_fallback_int!(I64Fallback, i64, "i64", "BigInteger", BigInteger);
    }

    #[test]
    fn active_enum_derive_unsigned_integers() {
        macro_rules! test_num_value_uint {
            ($ident: ident, $rs_type: expr, $db_type: expr, $col_def: ident) => {
                #[derive(Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
                #[sea_orm(rs_type = $rs_type, db_type = $db_type)]
                pub enum $ident {
                    #[sea_orm(num_value = 1)]
                    Big,
                    #[sea_orm(num_value = 0)]
                    Small,
                }

                test_uint!($ident, $rs_type, $db_type, $col_def);
            };
        }

        macro_rules! test_fallback_uint {
            ($ident: ident, $fallback_type: ident, $rs_type: expr, $db_type: expr, $col_def: ident) => {
                #[derive(Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
                #[sea_orm(rs_type = $rs_type, db_type = $db_type)]
                #[repr($fallback_type)]
                pub enum $ident {
                    Big = 1,
                    Small = 0,
                }

                test_uint!($ident, $rs_type, $db_type, $col_def);
            };
        }

        macro_rules! test_uint {
            ($ident: ident, $rs_type: expr, $db_type: expr, $col_def: ident) => {
                assert_eq!($ident::Big.to_value(), 1);
                assert_eq!($ident::Small.to_value(), 0);

                assert_eq!($ident::try_from_value(&1).ok(), Some($ident::Big));
                assert_eq!($ident::try_from_value(&0).ok(), Some($ident::Small));
                assert_eq!(
                    $ident::try_from_value(&2).err(),
                    Some(DbErr::Type(format!(
                        "unexpected value for {} enum: 2",
                        stringify!($ident)
                    )))
                );

                assert_eq!($ident::db_type(), ColumnType::$col_def.def());
            };
        }

        test_num_value_uint!(U8, "u8", "TinyInteger", TinyInteger);
        test_num_value_uint!(U16, "u16", "SmallInteger", SmallInteger);
        test_num_value_uint!(U32, "u32", "Integer", Integer);
        test_num_value_uint!(U64, "u64", "BigInteger", BigInteger);

        test_fallback_uint!(U8Fallback, u8, "u8", "TinyInteger", TinyInteger);
        test_fallback_uint!(U16Fallback, u16, "u16", "SmallInteger", SmallInteger);
        test_fallback_uint!(U32Fallback, u32, "u32", "Integer", Integer);
        test_fallback_uint!(U64Fallback, u64, "u64", "BigInteger", BigInteger);
    }
}
