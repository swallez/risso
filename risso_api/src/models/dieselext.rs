#![allow(proc_macro_derive_resolution_fallback)]

use diesel::backend::Backend;
use diesel::deserialize::{self, FromSql};
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Double;
use chrono::prelude::*;

use std;
use std::io::Write;

/// A wrapper around Chrono's `DataTime<Utc>` to read the ISSO database, that encodes dates using
/// a double containing fractional seconds since the Epoch (similar to what JavaScript does)

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, FromSqlRow, AsExpression)]
pub struct FloatDateTime(pub DateTime<Utc>);

impl FloatDateTime {
    fn from_f64(f: f64) -> FloatDateTime {
        FloatDateTime(Utc.from_utc_datetime(&NaiveDateTime::from_timestamp(
            f as i64,
            // Round to microseconds to have exact roundtrip (using nanos has some rounding errors)
            ((f.fract() * 1000_000.0).round() as u32) * 1000)
        ))
    }

    fn to_f64(&self) -> f64 {
        (self.0.timestamp() as f64) + (self.0.nanosecond() as f64 / 1_000_000_000.0)
    }
}

impl<DB> ToSql<Double, DB> for FloatDateTime
    where
        f64: ToSql<Double, DB>,
        DB: Backend,
{
    fn to_sql<W: Write>(&self, out: &mut Output<W, DB>) -> serialize::Result {
        let f = self.to_f64();
        ToSql::<Double, DB>::to_sql(&f, out)
    }
}

impl<DB> FromSql<Double, DB> for FloatDateTime
    where
        f64: FromSql<Double, DB>,
        DB: Backend,
{
    fn from_sql(value: Option<&<DB as Backend>::RawValue>) -> deserialize::Result<Self> {
        let f64_value = <f64 as FromSql<Double, DB>>::from_sql(value)?;
        Ok(FloatDateTime::from_f64(f64_value))
    }
}

//----- New type links to original type
// Could be automated using https://github.com/JelteF/derive_more

impl From<DateTime<Utc>> for FloatDateTime {
    fn from(dt: DateTime<Utc>) -> FloatDateTime {
        FloatDateTime(dt)
    }
}

impl AsRef<DateTime<Utc>> for FloatDateTime {
    fn as_ref(&self) -> &DateTime<Utc> {
        &self.0
    }
}

impl AsMut<DateTime<Utc>> for FloatDateTime {
    fn as_mut(&mut self) -> &mut DateTime<Utc> {
        &mut self.0
    }
}

impl std::ops::Deref for FloatDateTime {
    type Target = DateTime<Utc>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for FloatDateTime {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

//--------------------------------------------------------------------------------------------------
// Additional Diesel "DoubleTime" type that would map to a Chrono time that would avoid the
// newtype wrapper. Work in progress.

//#[derive(SqlType)]
//#[sqlite_type = "Double"]
//pub struct DoubleTime;
//
//impl<DB> ToSql<DoubleTime, DB> for FloatDateTime
//    where
//        f64: ToSql<Double, DB>,
//        DB: Backend,
//{
//    fn to_sql<W: Write>(&self, out: &mut Output<W, DB>) -> serialize::Result {
//        let f = self.to_f64();
//        ToSql::<Double, DB>::to_sql(&f, out)
//    }
//}
//
//impl<DB> FromSql<DoubleTime, DB> for FloatDateTime
//    where
//        f64: FromSql<DoubleTime, DB>,
//        DB: Backend,
//{
//    fn from_sql(value: Option<&<DB as Backend>::RawValue>) -> deserialize::Result<Self> {
//        let f64_value = <f64 as FromSql<Double, DB>>::from_sql(value)?;
//        Ok(FloatDateTime::from_f64(f64_value))
//    }
//}
//


#[cfg(test)]
mod tests {

    use chrono::prelude::*;
    use super::FloatDateTime;

    #[test]
    fn to_from_f64() {
        let n = Utc::now();
        let f = FloatDateTime(n).to_f64();

        let n2 = FloatDateTime::from_f64(f).0;
        let f2 = FloatDateTime(n2).to_f64();

        // debug code -- helped find the nanosecond rounding issue
//        let x = n.nanosecond() as f64 / 1_000_000_000.0;
//        let y = (x.fract() * 1000_000_000.0) as u32;
//        println!("{} / {}", x, y);
//
//        println!("n  = {:?} - {} / {}", n, n.timestamp(), n.nanosecond());
//        println!("n2 = {:?} - {} / {}", n2, n2.timestamp(), n2.nanosecond());
//        println!("n  = {:?} / {}", n, f);
//        println!("n2 = {:?} / {}", n2, f2);

        assert_eq!(f, f2);
        assert_eq!(n, n2);
    }
}
