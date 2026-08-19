pub mod codes;
pub mod date_time;
pub mod http;
pub mod resources;
pub mod types;

#[cfg(feature = "r4")]
pub mod r4;
#[cfg(feature = "r5")]
pub mod r5;

#[macro_export]
macro_rules! resource {
    ([$($resource:ident),*$(,)?]) => {
        #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
        #[serde(tag = "resourceType")]
        pub enum Resource {
            $(
            $resource($resource),
            )*
        }
    };
}

#[macro_export]
macro_rules! type_struct {
    (@ $resource:ident { } -> ($($fields:tt)*)) => {
        pastey::paste! {
            #[derive(serde::Serialize, serde::Deserialize, derive_builder::Builder, Debug, Clone, PartialEq)]
            #[serde(rename_all = "camelCase")]
            pub struct $resource {
                $($fields)*
            }

            impl $resource {
                pub fn builder() -> [<$resource Builder>] {
                    [<$resource Builder>]::default()
                }
            }
        }
    };

    ( @ $resource:ident { $(#[$attributes:meta])* @nodefault $vis:vis $key:ident: Option<$value:ty> $(, $($parameters:tt)*)? } -> ($($processed:tt)*) ) => (
        $crate::type_struct!(@ $resource { $($($parameters)*)? } -> (
            $($processed)*
            #[serde(skip_serializing_if = "Option::is_none")]
            #[builder(setter(into))]
            $(#[$attributes])*
            $vis $key: Option<$value>,
        ));
    );

    ( @ $resource:ident { $(#[$attributes:meta])* $vis:vis $key:ident: Option<$value:ty> $(, $($parameters:tt)*)? } -> ($($processed:tt)*) ) => (
        $crate::type_struct!(@ $resource { $($($parameters)*)? } -> (
            $($processed)*
            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[builder(default, setter(into))]
            $(#[$attributes])*
            $vis $key: Option<$value>,
        ));
    );

    ( @ $resource:ident { $(#[$attributes:meta])* @nodefault $vis:vis $key:ident: Vec<$value:ty> $(, $($parameters:tt)*)? } -> ($($processed:tt)*) ) => (
        $crate::type_struct!(@ $resource { $($($parameters)*)? } -> (
            $($processed)*
            #[serde(skip_serializing_if = "Vec::is_empty")]
            $(#[$attributes])*
            $vis $key: Vec<$value>,
        ));
    );

    ( @ $resource:ident { $(#[$attributes:meta])* $vis:vis $key:ident: Vec<$value:ty> $(, $($parameters:tt)*)? } -> ($($processed:tt)*) ) => (
        $crate::type_struct!(@ $resource { $($($parameters)*)? } -> (
            $($processed)*
            #[serde(default, skip_serializing_if = "Vec::is_empty")]
            #[builder(default)]
            $(#[$attributes])*
            $vis $key: Vec<$value>,
        ));
    );

    ( @ $resource:ident { $(#[$attributes:meta])* $vis:vis $key:ident: String $(, $($parameters:tt)*)? } -> ($($processed:tt)*) ) => (
        $crate::type_struct!(@ $resource { $($($parameters)*)? } -> (
            $($processed)*
            #[builder(setter(into))]
            $(#[$attributes])*
            $vis $key: String,
        ));
    );

    ( @ $resource:ident { $(#[$attributes:meta])* $vis:vis $key:ident: $value:ty $(, $($parameters:tt)*)? } -> ($($processed:tt)*) ) => (
        $crate::type_struct!(@ $resource { $($($parameters)*)? } -> (
            $($processed)*
            $(#[$attributes])*
            $vis $key: $value,
        ));
    );

    ( $resource:ident { $($body:tt)* } ) => (
        $crate::type_struct!(@ $resource { $($body)* } -> ());
    );
}
