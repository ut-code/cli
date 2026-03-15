use serde::Deserialize;
use worker::*;

// ---- QueueDO ----------------------------------------------------------------
// Singleton Durable Object that tracks waiting programmers.
// Persistent storage: Map<room_id, label> via DO key-value storage.

#[durable_object]
pub struct QueueDO {
    state: State,
    #[allow(dead_code)]
    env: Env,
}

impl DurableObject for QueueDO {
    fn new(state: State, env: Env) -> Self {
        Self { state, env }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        let url = req.url()?;
        let path = url.path();

        match req.method() {
            // GET /queue — return { roomId: label, ... }
            Method::Get if path == "/queue" => {
                let map = self.state.storage().list().await?;
                // js_sys::Map -> serde_json::Map via entries iterator
                let mut obj = serde_json::Map::new();
                map.for_each(&mut |v, k| {
                    if let (Some(key), Some(val)) = (k.as_string(), v.as_string()) {
                        obj.insert(key, serde_json::Value::String(val));
                    }
                });
                Response::from_json(&obj)
            }

            // POST /queue — register { roomId, label }
            Method::Post if path == "/queue" => {
                let mut req = req;
                let body: RegisterBody = req.json().await?;
                self.state.storage().put(&body.room_id, &body.label).await?;
                Response::empty().map(|r| r.with_status(204))
            }

            // DELETE /queue/:roomId — deregister
            Method::Delete if path.starts_with("/queue/") => {
                let room_id = &path["/queue/".len()..];
                self.state.storage().delete(room_id).await?;
                Response::empty().map(|r| r.with_status(204))
            }

            _ => Response::error("Not found", 404),
        }
    }
}

#[derive(Deserialize)]
struct RegisterBody {
    #[serde(rename = "roomId")]
    room_id: String,
    label: String,
}

// ---- RoomSession ------------------------------------------------------------
// Per-room Durable Object that relays messages between programmer and client.

#[durable_object]
pub struct RoomSession {
    state: State,
    #[allow(dead_code)]
    env: Env,
}

impl DurableObject for RoomSession {
    fn new(state: State, env: Env) -> Self {
        Self { state, env }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        let url = req.url()?;
        let path = url.path();
        let role = if path.ends_with("/programmer") {
            "programmer"
        } else {
            "client"
        };

        let pair = WebSocketPair::new()?;
        self.state.accept_websocket_with_tags(&pair.server, &[role]);
        Response::from_websocket(pair.client)
    }

    async fn websocket_message(
        &self,
        ws: WebSocket,
        message: WebSocketIncomingMessage,
    ) -> Result<()> {
        let text = match &message {
            WebSocketIncomingMessage::String(s) => s.clone(),
            WebSocketIncomingMessage::Binary(_) => return Ok(()),
        };

        let tags = self.state.get_tags(&ws);
        if tags.iter().any(|t| t == "client") {
            for p in self.state.get_websockets_with_tag("programmer") {
                p.send_with_str(&text)?;
            }
        } else {
            for c in self.state.get_websockets_with_tag("client") {
                c.send_with_str(&text)?;
            }
        }
        Ok(())
    }

    async fn websocket_close(
        &self,
        ws: WebSocket,
        _code: usize,
        _reason: String,
        _was_clean: bool,
    ) -> Result<()> {
        let tags = self.state.get_tags(&ws);
        if tags.iter().any(|t| t == "client") {
            for p in self.state.get_websockets_with_tag("programmer") {
                let _ = p.close::<&str>(None, None);
            }
        } else {
            for c in self.state.get_websockets_with_tag("client") {
                let _ = c.close::<&str>(None, None);
            }
        }
        Ok(())
    }
}

// ---- Fetch handler ----------------------------------------------------------

#[derive(Deserialize)]
struct QueueRegisterRequest {
    label: String,
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let url = req.url()?;
    let path = url.path();

    let queue_stub = || -> Result<Stub> {
        let ns = env.durable_object("QUEUE")?;
        let id = ns.id_from_name("global")?;
        id.get_stub()
    };

    let path_ref: &str = &path;
    match (req.method(), path_ref) {
        // GET /queue
        (Method::Get, "/queue") => queue_stub()?.fetch_with_str("http://do/queue").await,

        // POST /queue — create room, register programmer
        (Method::Post, "/queue") => {
            let mut req = req;
            let body: QueueRegisterRequest = req.json().await?;

            let room_id = {
                let ns = env.durable_object("ROOM")?;
                ns.unique_id()?.to_string()
            };

            let register_body = serde_json::json!({
                "roomId": room_id,
                "label": body.label,
            })
            .to_string();

            queue_stub()?
                .fetch_with_request(Request::new_with_init(
                    "http://do/queue",
                    RequestInit::new()
                        .with_method(Method::Post)
                        .with_body(Some(wasm_bindgen::JsValue::from_str(&register_body))),
                )?)
                .await?;

            Response::from_json(&serde_json::json!({ "roomId": room_id }))
        }

        // DELETE /queue/:roomId
        (Method::Delete, path) if path.starts_with("/queue/") => {
            let room_id = &path["/queue/".len()..];
            queue_stub()?
                .fetch_with_request(Request::new_with_init(
                    &format!("http://do/queue/{}", room_id),
                    RequestInit::new().with_method(Method::Delete),
                )?)
                .await
        }

        // WS /rooms/:id/programmer
        (Method::Get, path) if path.starts_with("/rooms/") && path.ends_with("/programmer") => {
            let room_id = path
                .trim_start_matches("/rooms/")
                .trim_end_matches("/programmer");
            let ns = env.durable_object("ROOM")?;
            ns.id_from_string(room_id)?
                .get_stub()?
                .fetch_with_request(req)
                .await
        }

        // WS /rooms/:id/client
        (Method::Get, path) if path.starts_with("/rooms/") && path.ends_with("/client") => {
            let room_id = path
                .trim_start_matches("/rooms/")
                .trim_end_matches("/client");
            let ns = env.durable_object("ROOM")?;
            ns.id_from_string(room_id)?
                .get_stub()?
                .fetch_with_request(req)
                .await
        }

        _ => Response::error("Not found", 404),
    }
}
