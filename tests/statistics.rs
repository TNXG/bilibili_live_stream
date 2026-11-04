use bili_live::model::LiveStats;
use serde_json::json;

#[test]
fn statistics_accept_integer_strings_and_default_absent_counters() {
    let stats: LiveStats = serde_json::from_value(json!({
        "AddFans":"2", "DanmuNum":3, "LiveTime":"3600", "HamsterRmb":4,
        "MaxOnline":"5", "NewFansClub":6, "WatchedCount":"7"
    }))
    .unwrap();
    assert_eq!((stats.fans, stats.danmaku, stats.seconds), (2, 3, 3600));
    assert_eq!(
        (
            stats.revenue,
            stats.max_online,
            stats.fans_club,
            stats.viewers
        ),
        (4.0, 5, 6, 7)
    );
    let empty: LiveStats = serde_json::from_value(json!({})).unwrap();
    assert_eq!(
        (empty.fans, empty.danmaku, empty.seconds, empty.viewers),
        (0, 0, 0, 0)
    );
    assert_eq!(empty.revenue, 0.0);
}

#[test]
fn revenue_accepts_fractional_yuan_as_number_or_string() {
    for value in [json!(12.34), json!("12.34")] {
        let stats: LiveStats = serde_json::from_value(json!({"HamsterRmb": value})).unwrap();
        assert!((stats.revenue - 12.34).abs() < f64::EPSILON);
    }
}

#[test]
fn statistics_do_not_silently_coerce_invalid_counters() {
    for value in [
        json!(1.5),
        json!("invalid"),
        json!(null),
        json!(true),
        json!(u64::MAX),
    ] {
        assert!(serde_json::from_value::<LiveStats>(json!({"LiveTime":value})).is_err());
    }
}
