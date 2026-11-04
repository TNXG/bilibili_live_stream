use super::Api;
use crate::error::{BiliLiveError, Result};
use crate::model::session::parse_protocols;
use crate::model::{Area, AreaGroup, Credentials, LiveSession, LiveStats, PushProtocol};
use serde::Deserialize;
use serde_json::Value;

/// Build advertised by the decompiled 直播姬 client that the speed-test endpoint expects.
const SPEED_TEST_BUILD: &str = "7120300";

#[derive(Deserialize)]
struct SpeedTestInfo {
    #[serde(default)]
    nodes: Vec<SpeedNode>,
}

#[derive(Deserialize)]
struct SpeedNode {
    #[serde(default)]
    protocol: String,
    #[serde(default)]
    addr: String,
    #[serde(default)]
    ip: String,
}

impl Api {
    pub fn areas(&self) -> Result<Vec<AreaGroup>> {
        self.data(self.client.get(self.live_url("/room/v1/Area/getList")))
    }

    pub fn recent_area(&self, c: &Credentials) -> Result<Option<Area>> {
        let areas: Vec<Area> = self.data(
            self.authenticated(
                self.client
                    .get(self.live_url("/room/v1/Area/getMyChooseArea"))
                    .query(&[("roomid", c.room_id)]),
                c,
            )?,
        )?;
        Ok(areas.into_iter().find(|area| area.id > 0))
    }

    pub fn start(&self, c: &Credentials, area_id: u64, request_srt: bool) -> Result<LiveSession> {
        if area_id == 0 {
            return Err(BiliLiveError::Parse("请选择有效分区".into()));
        }
        let mut fields = vec![
            ("room_id", c.room_id.to_string()),
            ("area_v2", area_id.to_string()),
            ("csrf", c.csrf_token.clone()),
            ("platform", "pc_link".into()),
        ];
        // The server only attaches an SRT line when the request names both the
        // protocol and a matching speed-test node; the protocol hint alone is ignored.
        if request_srt {
            match self.srt_recommendation(c, area_id) {
                Ok(Some(ip)) => {
                    fields.push(("recommend_protocol", "srt".into()));
                    fields.push(("recommend_ip", ip));
                }
                Ok(None) => {
                    log::warn!("未找到可用的 SRT 节点，本次仅获取默认线路");
                }
                Err(error) => log::warn!("获取 SRT 节点失败：{error}"),
            }
        }
        // code=0 means active, regardless of missing or malformed optional metadata.
        self.data(
            self.authenticated(
                self.client
                    .post(self.live_url("/room/v1/Room/startLive"))
                    .header("platform", "web_electron_link")
                    .form(&fields),
                c,
            )?,
        )
    }

    /// IP of a speed-test node that advertises the private RTMP-over-SRT scheme.
    /// `recommend_protocol` alone is ignored by the server; `recommend_ip` must
    /// name a node from this response for the SRT line to be attached.
    fn srt_recommendation(&self, c: &Credentials, area_id: u64) -> Result<Option<String>> {
        let info: SpeedTestInfo =
            self.data(
                self.authenticated(
                    self.client
                        .get(self.live_url(
                            "/xlive/app-blink/v1/upStreamConfig/GetUpStreamSpeedTestInfo",
                        ))
                        .query(&[
                            ("room_id", c.room_id.to_string()),
                            ("platform", "pc_link".to_owned()),
                            ("build", SPEED_TEST_BUILD.to_owned()),
                            ("area_v2_id", area_id.to_string()),
                            ("free_tab", "0".to_owned()),
                        ]),
                    c,
                )?,
            )?;
        Ok(info
            .nodes
            .into_iter()
            .find(|node| {
                (node.protocol.eq_ignore_ascii_case("srt")
                    || node.protocol.eq_ignore_ascii_case("rtmpsrt"))
                    && !node.ip.is_empty()
                    && node.addr.to_ascii_lowercase().starts_with("rtmpsrt://")
            })
            .map(|node| node.ip))
    }

    pub fn refresh(&self, c: &Credentials, area_id: u64) -> Result<Vec<PushProtocol>> {
        let envelope = self.request(
            self.authenticated(
                self.client
                    .get(self.live_url("/xlive/app-blink/v1/live/GetUpStreamRtmp"))
                    .query(&[("room_id", c.room_id), ("area_v2", area_id)]),
                c,
            )?,
        )?;
        // Observed with cookie-only auth: this App endpoint rejects the request
        // instead of returning protocols, so `r` cannot discover new SRT lines.
        if envelope.code == -400 {
            return Err(BiliLiveError::Api(
                "刷新接口返回 -400（App 侧鉴权接口，Cookie 无法刷新）；SRT 以开播响应为准".into(),
            ));
        }
        let data: Value = envelope.decode()?;
        Ok(parse_protocols(&data))
    }

    pub fn stop(&self, c: &Credentials) -> Result<()> {
        self.request(
            self.authenticated(
                self.client
                    .post(self.live_url("/room/v1/Room/stopLive"))
                    .form(&[
                        ("room_id", c.room_id.to_string()),
                        ("csrf", c.csrf_token.clone()),
                        ("platform", "web_electron_link".into()),
                    ]),
                c,
            )?,
        )?
        .checked()?;
        Ok(())
    }

    pub fn stats(&self, c: &Credentials, live_key: &str) -> Result<LiveStats> {
        if live_key.is_empty() {
            return Err(BiliLiveError::Api("暂无本场统计数据".into()));
        }
        self.data(
            self.authenticated(
                self.client
                    .get(self.live_url("/xlive/app-blink/v1/live/StopLiveData"))
                    .query(&[("live_key", live_key)]),
                c,
            )?,
        )
    }
}
