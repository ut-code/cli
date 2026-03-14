import { Hono } from "hono";
import { RoomSession } from "./room";

export { RoomSession };

type Bindings = { ROOM: DurableObjectNamespace };

const app = new Hono<{ Bindings: Bindings }>();

app.post("/rooms", (c) => {
  const id = c.env.ROOM.newUniqueId();
  return c.json({ roomId: id.toString() });
});

app.get("/rooms/:id/user", (c) => {
  const id = c.env.ROOM.idFromString(c.req.param("id"));
  const stub = c.env.ROOM.get(id);
  return stub.fetch(c.req.raw);
});

app.get("/rooms/:id/programmer", (c) => {
  const id = c.env.ROOM.idFromString(c.req.param("id"));
  const stub = c.env.ROOM.get(id);
  return stub.fetch(c.req.raw);
});

export default app;
