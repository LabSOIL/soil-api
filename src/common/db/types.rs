// use geozero::wkb::{FromWkb, WkbDialect};
// use geozero::{CoordDimensions, GeomProcessor, GeozeroGeometry};
// use sea_orm::TryGetable;
// use std::io::Read;

// #[derive(Debug, PartialEq, Default, Clone)]
// pub struct PointZ {
//     x: f64,
//     y: f64,
//     z: f64,
// }

// impl GeomProcessor for PointZ {
//     fn dimensions(&self) -> CoordDimensions {
//         CoordDimensions::xyz()
//     }
//     fn coordinate(
//         &mut self,
//         x: f64,
//         y: f64,
//         z: Option<f64>,
//         _m: Option<f64>,
//         _t: Option<f64>,
//         _tm: Option<u64>,
//         _idx: usize,
//     ) -> geozero::error::Result<()> {
//         self.x = x;
//         self.y = y;
//         self.z = z.unwrap_or(0.0);
//         Ok(())
//     }
// }

// impl GeozeroGeometry for PointZ {
//     fn process_geom<P: GeomProcessor>(
//         &self,
//         processor: &mut P,
//     ) -> Result<(), geozero::error::GeozeroError> {
//         processor.point_begin(0)?;
//         processor.coordinate(self.x, self.y, Some(self.z), None, None, None, 0)?;
//         processor.point_end(0)
//     }
//     fn dims(&self) -> CoordDimensions {
//         CoordDimensions::xyz()
//     }
// }

// impl FromWkb for PointZ {
//     fn from_wkb<R: Read>(rdr: &mut R, dialect: WkbDialect) -> geozero::error::Result<Self> {
//         let mut pt = PointZ::default();
//         geozero::wkb::process_wkb_type_geom(rdr, &mut pt, dialect)?;
//         Ok(pt)
//     }
// }

// impl TryGetable for PointZ {
//     fn try_get(
//         res: &sea_orm::QueryResult,
//         index: &str,
//         column: &str,
//     ) -> Result<Self, sea_orm::TryGetError> {
//         let wkb = res.try_get_by::<_, Vec<u8>>(index)?;
//         let mut rdr = std::io::Cursor::new(wkb);
//         PointZ::from_wkb(&mut rdr, WkbDialect::Ewkb)
//             .map_err(|e| sea_orm::TryGetError::Conversion(e.to_string()))
//     }

//     fn try_get_by<I: sea_orm::ColIdx>(
//         res: &sea_orm::QueryResult,
//         index: I,
//     ) -> Result<Self, sea_orm::TryGetError> {
//         let wkb = res.try_get_by::<I, Vec<u8>>(index)?;
//         let mut rdr = std::io::Cursor::new(wkb);
//         PointZ::from_wkb(&mut rdr, WkbDialect::Ewkb)
//             .map_err(|e| sea_orm::TryGetError::Conversion(e.to_string()))
//     }

//     fn try_get_by_index(
//         res: &sea_orm::QueryResult,
//         index: usize,
//     ) -> Result<Self, sea_orm::TryGetError> {
//         let wkb = res.try_get_by_index::<Vec<u8>>(index)?;
//         let mut rdr = std::io::Cursor::new(wkb);
//         PointZ::from_wkb(&mut rdr, WkbDialect::Ewkb)
//             .map_err(|e| sea_orm::TryGetError::Conversion(e.to_string()))
//     }
// }
