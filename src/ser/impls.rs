use std::borrow::Cow;
use std::collections::{btree_map, BTreeMap};
use std::slice;

use private;
use ser::{Fragment, Map, Seq, Serialize};

impl Serialize for () {
    fn begin(&self) -> Fragment {
        Fragment::Null
    }
}

impl Serialize for bool {
    fn begin(&self) -> Fragment {
        Fragment::Bool(*self)
    }
}

impl Serialize for str {
    fn begin(&self) -> Fragment {
        Fragment::Str(Cow::Borrowed(self))
    }
}

impl Serialize for String {
    fn begin(&self) -> Fragment {
        Fragment::Str(Cow::Borrowed(self))
    }
}

macro_rules! unsigned {
    ($ty:ident) => {
        impl Serialize for $ty {
            fn begin(&self) -> Fragment {
                Fragment::U64(*self as u64)
            }
        }
    };
}
unsigned!(u8);
unsigned!(u16);
unsigned!(u32);
unsigned!(u64);

macro_rules! signed {
    ($ty:ident) => {
        impl Serialize for $ty {
            fn begin(&self) -> Fragment {
                Fragment::I64(*self as i64)
            }
        }
    };
}
signed!(i8);
signed!(i16);
signed!(i32);
signed!(i64);

macro_rules! float {
    ($ty:ident) => {
        impl Serialize for $ty {
            fn begin(&self) -> Fragment {
                Fragment::F64(*self as f64)
            }
        }
    };
}
float!(f32);
float!(f64);

impl<'a, T: ?Sized + Serialize> Serialize for &'a T {
    fn begin(&self) -> Fragment {
        (**self).begin()
    }
}

impl<T: ?Sized + Serialize> Serialize for Box<T> {
    fn begin(&self) -> Fragment {
        (**self).begin()
    }
}

impl<T: Serialize> Serialize for Option<T> {
    fn begin(&self) -> Fragment {
        match self {
            Some(some) => some.begin(),
            None => Fragment::Null,
        }
    }
}

impl<A: Serialize, B: Serialize> Serialize for (A, B) {
    fn begin(&self) -> Fragment {
        struct TupleStream<'a> {
            first: &'a Serialize,
            second: &'a Serialize,
            state: usize,
        }

        impl<'a> Seq for TupleStream<'a> {
            fn next(&mut self) -> Option<&Serialize> {
                let state = self.state;
                self.state += 1;
                match state {
                    0 => Some(self.first),
                    1 => Some(self.second),
                    _ => None,
                }
            }
        }

        Fragment::Seq(Box::new(TupleStream {
            first: &self.0,
            second: &self.1,
            state: 0,
        }))
    }
}

impl<T: Serialize> Serialize for [T] {
    fn begin(&self) -> Fragment {
        private::stream_slice(self)
    }
}

impl<T: Serialize> Serialize for Vec<T> {
    fn begin(&self) -> Fragment {
        private::stream_slice(self)
    }
}

impl<K: ToString, V: Serialize> Serialize for BTreeMap<K, V> {
    fn begin(&self) -> Fragment {
        private::stream_btree_map(self)
    }
}

impl private {
    pub fn stream_slice<T: Serialize>(slice: &[T]) -> Fragment {
        struct SliceStream<'a, T: 'a>(slice::Iter<'a, T>);

        impl<'a, T: Serialize> Seq for SliceStream<'a, T> {
            fn next(&mut self) -> Option<&Serialize> {
                let element = self.0.next()?;
                Some(element)
            }
        }

        Fragment::Seq(Box::new(SliceStream(slice.iter())))
    }

    pub fn stream_btree_map<K: ToString, V: Serialize>(map: &BTreeMap<K, V>) -> Fragment {
        struct BTreeMapStream<'a, K: 'a, V: 'a>(btree_map::Iter<'a, K, V>);

        impl<'a, K: ToString, V: Serialize> Map for BTreeMapStream<'a, K, V> {
            fn next(&mut self) -> Option<(Cow<str>, &Serialize)> {
                let (k, v) = self.0.next()?;
                Some((Cow::Owned(k.to_string()), v as &Serialize))
            }
        }

        Fragment::Map(Box::new(BTreeMapStream(map.iter())))
    }
}
