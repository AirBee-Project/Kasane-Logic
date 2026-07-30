/// 時間軸の生の2進セルにおけるズームレベルを表す型。
///
/// [`ZoomLevel`](crate::spatial_id::zoom_level::ZoomLevel)（空間軸、`LIMIT=30`）と同じ土台を
/// 共有する `ZoomLevel<62>` の別名。1セルが最大`2^62`秒（全期間）〜1秒を表す。
///
/// ```
/// # use kasane_logic::spatial_id::temporal_id::zoom_level::TZoomLevel;
/// let z = TZoomLevel::new(5).unwrap();
/// assert_eq!(z.get(), 5);
/// ```
pub type TZoomLevel = crate::spatial_id::zoom_level::Zoom<62>;
